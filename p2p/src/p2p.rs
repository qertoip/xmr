use std::net::SocketAddr;
use std::sync::Arc;

use failure::Error;
use futures_cpupool::CpuPool;
use futures::{Future, finished};
use tokio_core::reactor::{Handle, Remote};
use rand::OsRng;

use db::Store;

use config::Config;
use protocol::PeerId;
use net::{connect, ConnectionCounter};
use levin::Command;
use cryptonote::CoreSyncData;
use protocol::BasicNodeData;
use protocol::handshake::CryptoNoteHandshake;

pub type BoxedEmptyFuture = Box<Future<Item=(), Error=()> + Send>;

pub struct Context {
    connection_counter: ConnectionCounter,
    remote: Remote,
    pool: CpuPool,
    pub(crate) config: Config,
    pub(crate) peer_id: PeerId,
}

impl Context {
    pub fn new(pool_handle: CpuPool,
               remote: Remote,
               config: Config) -> Context {
        let mut rng = OsRng::new().expect("Cannot open OS random.");
        let peer_id = PeerId::random(&mut rng);
        Context {
            // TODO: Add a cfg for max inbound/outbound connections
            connection_counter: ConnectionCounter::new(5, 5),
            remote: remote,
            pool: pool_handle,
            config,
            peer_id,
        }
    }

    pub fn connect(context: Arc<Context>,
                   address: SocketAddr,
                   req: <CryptoNoteHandshake as Command>::Request) {
        context.connection_counter.note_new_outbound_connection();
        context.remote.clone().spawn(move |handle| {
            context.pool.clone().spawn(Self::connect_future(context.clone(), handle, address, req))
        })
    }

    pub fn connect_future(context: Arc<Context>,
                          handle: &Handle,
                          address: SocketAddr,
                          req: <CryptoNoteHandshake as Command>::Request) -> BoxedEmptyFuture {
        let connection = connect(&address, handle, context.clone(), req);
        Box::new(connection.then(move |result| {
            match result {
                Ok(response) => {
                    match response {
                        Ok((_stream, response)) => panic!("ok"),
                        Err(e) => {
                            context.connection_counter.note_close_outbound_connection();
                            panic!("{:?}", e);
                        }
                    }
                },
                Err(e) => {
                    context.connection_counter.note_close_outbound_connection();
                    panic!("{:?}", e);
                }
            }

            finished(())
        }))
    }

    fn basic_node_data(&self) -> BasicNodeData {
        BasicNodeData {
            network_id: self.config.network_id.into(),
            local_time: 0,
            my_port: self.config.listen_port,
            peer_id: self.peer_id,
        }
    }
}

pub struct P2P {
    event_loop_handle: Handle,
    context: Arc<Context>,
    _pool: CpuPool,
}

impl P2P {
    pub fn new(config: Config, handle: Handle) -> P2P {
        let pool = CpuPool::new(config.threads);
        let remote = handle.remote().clone();
        P2P {
            event_loop_handle: handle,
            context: Arc::new(Context::new(pool.clone(), remote, config.clone())),
            _pool: pool,
        }
    }

    pub fn run<S>(&self, store: &S) -> Result<(), Error> where S: Store {
        type Request = <CryptoNoteHandshake as Command>::Request;

        for addr in self.context.config.peers.iter() {
            let req = Request {
                node_data: self.context.basic_node_data(),
                payload_data: core_sync_data(store)
            };

            Context::connect(self.context.clone(), addr.clone(), req)
        }

        Ok(())
    }
}

fn core_sync_data<S>(store: &S) -> CoreSyncData where S: Store {
    let best_block = store.best_block();
    CoreSyncData {
        current_height: best_block.height,
        cumulative_difficulty: 0,
        top_id: best_block.id,
        // TODO: ideal hard fork verion.
        top_version: 0,
    }
}
