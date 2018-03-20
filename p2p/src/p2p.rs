use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

use failure::Error;

use futures::Future;
use futures_cpupool::CpuPool;
use tokio_core::reactor::{Handle, Remote};

use parking_lot::RwLock;

use storage::SharedStore;

use levin::net::{IoHandler, IoHandlerRef, TcpServer, Commands, ConnectionHandler as ConnectionHandlerTrait, ConnectionHandlerRef, connect as levin_connect};

use portable_storage::{Section, from_section, to_section};

use config::Config;

use net::{ConnectionCounter, ConnectionType, PeerContext};
use protocol::{LocalSyncNodeRef, OutboundSync};

use types::BasicNodeData;
use types::cn::CoreSyncData;
use types::cmd::{Handshake, HandshakeRequest, HandshakeResponse, Ping, PingResponse,
                 RequestSupportFlags, SupportFlagsResponse, TimedSync,
                 TimedSyncRequest, TimedSyncResponse};

use utils::Peerlist;

pub type BoxedEmptyFuture = Box<Future<Item = (), Error = ()> + Send>;

pub struct Context {
    remote: Remote,
    _pool: CpuPool,
    config: Config,
    connection_counter: ConnectionCounter,
    store: SharedStore,
    pub(crate) command_streams: RwLock<HashMap<SocketAddr, Commands>>,
    peerlist: RwLock<Peerlist>,
    local_sync_node: LocalSyncNodeRef,
}

impl Context {
    pub fn new(remote: Remote,
               pool: CpuPool,
               config: Config,
               store: SharedStore,
               local_sync_node: LocalSyncNodeRef) -> Context {
        let connection_counter = ConnectionCounter::new(config.in_peers, config.out_peers);

        let max_peers = config.in_peers + config.out_peers;
        let command_streams = RwLock::new(HashMap::with_capacity(max_peers as _));

        Context {
            remote,
            _pool: pool,
            config,
            connection_counter,
            store,
            command_streams,
            peerlist: RwLock::new(Peerlist::new()),
            local_sync_node,
        }
    }

    pub fn close(context: Arc<Context>, addr: &SocketAddr) {
        if let Some(command_stream) = context.command_streams.write().remove(addr) {
            command_stream.shutdown();
            context.connection_counter.note_close_connection(addr);
        }
    }

    pub fn spawn_server(context: Arc<Context>, io_handler: IoHandlerRef) {
        let addr = context.config
            .listen_port
            .map(|port| format!("127.0.0.1:{}", port))
            .unwrap_or(format!("127.0.0.1:{}", context.config.network.listen_port()))
            .parse()
            .unwrap();

        context
            .remote
            .clone()
            .spawn(move |handle| {
                // TODO: spawn this future on the threadpool.
                let connection_handler = ConnectionHandler::new(context.clone());
                TcpServer::bind(&addr, handle, io_handler, connection_handler)
                    .unwrap()
                    .run()
                    .map_err(|e| {
                        warn!("server io error: {}", e);
                        ()
                    })
            })
    }

    pub fn connect(context: Arc<Context>, addr: &SocketAddr, io_handler: IoHandlerRef) {
        let addr = addr.clone();
        context
            .remote
            .clone()
            .spawn(move |handle| {
                // TODO: on threadpool
                
                let commands = Commands::new();

                let request = to_section(&HandshakeRequest {
                    node_data: Context::basic_node_data(context.clone()),
                    payload_data: Context::core_sync_data(context.clone()),
                }).unwrap();

                commands.invoke::<Handshake, _>(request, {
                    let context = context.clone();
                    let addr = addr.clone();
                    move |response: Section| {
                        // TODO: handle errors
                        let response: HandshakeResponse = from_section(response).unwrap();

                        if response.node_data.peer_id == context.config.peer_id {
                            warn!("same peer id from address {}, disconnecting", addr);
                            Context::close(context.clone(), &addr);
                        }

                        let peer_context = PeerContext::new(context.clone(), addr.clone());
                        let outbound_sync = Arc::new(OutboundSync::new(peer_context));

                        let peer_id = response.node_data.peer_id;
                        let sync_data = response.payload_data;

                        context.local_sync_node.new_sync_connection(peer_id, &sync_data, outbound_sync);
                    }
                });

                context.command_streams.write().insert(addr.clone(), commands.clone());
                context.connection_counter.note_new_outbound_connection(addr.clone());
                // XXX: peerlist?

                levin_connect(&addr, handle, io_handler, commands)
                    .map_err(|e| {
                        warn!("connect io error: {}", e);
                        ()
                    })
            })
    }

    pub fn on_handshake(context: Arc<Context>,
                        addr: SocketAddr,
                        request: HandshakeRequest) -> Option<HandshakeResponse> {
        let network_id = request.node_data.network_id.0;
        if network_id != context.config.network.id() {
            info!("wrong network agen connected! id {}", network_id);
            Context::close(context.clone(), &addr);

            return None;
        }

        match context.connection_counter.connection_type(&addr) {
            Some(ConnectionType::Outbound) => {
                info!("handshake didn't came from inbound connection! address {}", addr);
                Context::close(context.clone(), &addr);

                return None;
            },
            None => unreachable!(),
            _ => {/* it's fine */}
        }


        // TODO: check for double handshake
        
        // TODO: update sync data.

        if context.config.peer_id != request.node_data.peer_id && request.node_data.my_port != 0 {
            // TODO: check if peer responds to ping and insert to context.peerlist
            unimplemented!();
        }

        let command_stream = context.command_streams.read().get(&addr).cloned().unwrap();
        command_stream.invoke::<RequestSupportFlags, _>(Section::new(), {
            |_response: Section| {
                // TODO: handle support flags.
                unimplemented!();
            }
        });

        Some(HandshakeResponse {
            node_data: Context::basic_node_data(context.clone()), 
            payload_data: Context::core_sync_data(context.clone()),
            local_peerlist: context.peerlist.read().stl_peerlist(),
        })
    }

    pub fn on_ping(context: Arc<Context>) -> PingResponse {
        PingResponse::new(context.config.peer_id)
    }

    pub fn on_request_support_flags() -> SupportFlagsResponse {
        SupportFlagsResponse::supported()
    }

    pub fn on_timed_sync(context: Arc<Context>,
                         _addr: SocketAddr,
                         _request: TimedSyncRequest) -> TimedSyncResponse {
        // TODO: handle request.payload_data

        TimedSyncResponse {
            local_time: Context::local_time(),
            payload_data: Context::core_sync_data(context.clone()),
            local_peerlist: context.peerlist.read().stl_peerlist(),
        }
    }

    fn io_handler(context: Arc<Context>) -> IoHandlerRef {
        let mut io_handler = IoHandler::with_capacity(12);

        io_handler.add_invokation::<Handshake, _>({
            let context = context.clone();
            move |addr: SocketAddr, request: Section| -> Result<Option<Section>, i32> {
                from_section(request)
                    .map_err(|_| -1)
                    .map(|request: HandshakeRequest| {
                        Context::on_handshake(context.clone(), addr, request)
                            .map(|res| to_section(&res).unwrap())
                    })
            }
        });

        io_handler.add_invokation::<Ping, _>({
            let context = context.clone();
            move |_: SocketAddr, _: Section| -> Result<Option<Section>, i32> {
                let res = Context::on_ping(context.clone());
                Ok(Some(to_section(&res).unwrap()))
            }
        });

        io_handler.add_invokation::<RequestSupportFlags, _>({
            move |_: SocketAddr, _: Section| -> Result<Option<Section>, i32> {
                let res = Context::on_request_support_flags();
                Ok(Some(to_section(&res).unwrap()))
            }
        });

        io_handler.add_invokation::<TimedSync, _>({
            let context = context.clone();
            move |addr: SocketAddr, request: Section| -> Result<Option<Section>, i32> {
                from_section(request)
                    .map(|request: TimedSyncRequest| {
                        let res = Context::on_timed_sync(context.clone(), addr, request);
                        Some(to_section(&res).unwrap())
                    })
                    .map_err(|_| -1)
            }
        });

        io_handler.to_ref()
    }

    pub fn basic_node_data(context: Arc<Context>) -> BasicNodeData {
        let my_port = if context.config.hide_my_port {
            0
        } else {
            context.config
                   .listen_port
                   .map(|p| p as u32)
                   .unwrap_or(context.config.network.listen_port() as u32)
        };

        BasicNodeData {
            network_id: context.config.network.id().into(),
            local_time: Context::local_time(),
            my_port,
            peer_id: context.config.peer_id,
        }
    }

    pub fn core_sync_data(context: Arc<Context>) -> CoreSyncData {
        let best_block = context.store.best_block();
        CoreSyncData {
            // TODO: cumulative difficulty?,
            cumulative_difficulty: 0,
            current_height: best_block.height,
            top_id: best_block.id,
            top_version: context.config.network.hard_forks().ideal_version(),
        }
    }

    fn local_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system time is behind unix epoch")
            .as_secs()
    }
}

pub struct P2P {
    config: Config,
    context: Arc<Context>,
}

impl P2P {
    pub fn new(config: Config,
               handle: Handle,
               store: SharedStore,
               local_sync_node: LocalSyncNodeRef) -> P2P {
        trace!("p2p config: {:?}", config);

        let pool = CpuPool::new(config.threads);
        let remote = handle.remote().clone();
        P2P {
            config: config.clone(),
            context: Arc::new(Context::new(remote, pool, config, store, local_sync_node)),
        }
    }

    pub fn run(&self) -> Result<(), Error> {
        let io_handler = Context::io_handler(self.context.clone());

        if !self.config.hide_my_port {
            info!("spawning the levin server.");
            Context::spawn_server(self.context.clone(), io_handler.clone())
        }

        for addr in self.config.peers.iter() {
            info!("connecting to {}", addr);
            Context::connect(self.context.clone(), addr, io_handler.clone())
        }

        Ok(())
    }
}

pub struct ConnectionHandler {
    context: Arc<Context>,
}

impl ConnectionHandler {
    pub fn new(context: Arc<Context>) -> ConnectionHandlerRef {
        Arc::new(ConnectionHandler { context })
    }
}

impl ConnectionHandlerTrait for ConnectionHandler {
    fn on_connect(&self, addr: SocketAddr, commands: Commands) {
        info!("new inbound connection from {}", addr);
        self.context.command_streams.write().insert(addr.clone(), commands);
        self.context.connection_counter.note_new_inbound_connection(addr.clone());
    }
}
