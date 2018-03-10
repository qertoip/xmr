use std::io;
use std::sync::Arc;
use std::net::SocketAddr;

use futures::{Future, Poll};
use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};

use uuid::Uuid;

use p2p::Context;
use config::P2P_SUPPORT_FLAGS;

use types::handshake::Handshake;
use types::request_support_flags::RequestSupportFlags;

use levin::{
    LevinError,
    Command,
    Invoke,
    Receive,
    Response as LevinResponse,
    invoke,
    receive,
    response,
};

pub type Request = <Handshake as Command>::Request;
pub type Response = <Handshake as Command>::Response;
type SupportFlagsResponse = <RequestSupportFlags as Command>::Response;

pub fn connect(address: &SocketAddr,
               handle: &Handle,
               context: Arc<Context>,
               request: Request) -> Connect {
    Connect {
        context,
        state: ConnectState::TcpConnect {
            future: TcpStream::connect(address, handle),
            request,
        }
    }
}

pub struct Connect {
    state: ConnectState,
    context: Arc<Context>,
}

enum ConnectState {
    TcpConnect {
        future: TcpStreamNew,
        request: Request,
    },
    InvokeHandshake {
        future: Invoke<TcpStream>,
    },
    ReceiveRequestSupportFlags {
        future: Receive<TcpStream, <RequestSupportFlags as Command>::Request>,
    },
    SendSupportFlags {
        future: LevinResponse<TcpStream>,
    },
    ReceiveHandshakeResponse {
        future: Receive<TcpStream, <Handshake as Command>::Response>,
    }
}

impl Future for Connect {
    type Item = (TcpStream, Result<Response, ConnectError>);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let next_state = match self.state {
                ConnectState::TcpConnect { ref mut future, ref request } => {
                    let stream = try_ready!(future.poll());
                    ConnectState::InvokeHandshake {
                        future: invoke::<Handshake, TcpStream>(stream, request),
                    }
                },
                ConnectState::InvokeHandshake { ref mut future } => {
                    let stream = try_ready!(future.poll());

                    ConnectState::ReceiveRequestSupportFlags {
                        future: receive(stream),
                    }
                },
                ConnectState::ReceiveRequestSupportFlags { ref mut future } => {
                    let (stream, request) = try_ready!(future.poll());
                    if let Err(e) = request {
                        return Ok((stream, Err(e.into())).into());
                    }

                    let res = SupportFlagsResponse {
                        support_flags: P2P_SUPPORT_FLAGS,
                    };

                    ConnectState::SendSupportFlags {
                        future: response::<TcpStream, RequestSupportFlags>(stream, res)
                    }
                },
                ConnectState::SendSupportFlags { ref mut future } => {
                    let stream = try_ready!(future.poll());

                    ConnectState::ReceiveHandshakeResponse {
                        future: receive(stream),
                    }
                },
                ConnectState::ReceiveHandshakeResponse { ref mut future } => {
                    let (stream, response) = try_ready!(future.poll());

                    let response = match response {
                        Ok((_, rsp)) => rsp,
                        Err(e) => return Ok((stream, Err(e.into())).into()),
                    };

                    if response.node_data.network_id.0 != self.context.config.network.id() {
                        return Ok((stream, Err(ConnectError::WrongNetwork(response.node_data.network_id.0))).into());
                    }
                    
                    if response.node_data.peer_id == self.context.peer_id {
                        return Ok((stream, Err(ConnectError::SamePeerId)).into());
                    }

                    return Ok((stream, Ok(response)).into());
                }
            };
            self.state = next_state;
        }
    }
}

#[derive(Debug)]
pub enum ConnectError {
    /// A levin error.
    LevinError(LevinError),
    /// Wrong network Id.
    WrongNetwork(Uuid),
    /// The peer has the same peer id, probably connected to self.
    SamePeerId,
}

impl From<LevinError> for ConnectError {
    fn from(e: LevinError) -> ConnectError {
        ConnectError::LevinError(e)
    }
}
