use websocket::{OwnedMessage,
    stream::sync::Splittable,
    sync::Server,
    client::sync::Client,
    server::{NoTlsAcceptor,
        sync::AcceptResult},
    receiver, sender};
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::sync::{mpsc,
                mpsc::{Sender, Receiver}};
use std::collections::HashMap;
use super::lobby::Lobby;
use super::backend_connection::BackendConnection;

pub type ClientReceiver = receiver::Reader<<TcpStream as Splittable>::Reader>;
pub type ClientSender = sender::Writer<<TcpStream as Splittable>::Writer>;

const PROTOCOL: &str = "tuesday";

pub type Token = u32;
pub type UserId = u32;

#[derive(Debug)]
pub enum GameServerError {
    BindError(std::io::Error),
    HandshakeRequestError,
    InvalidProtocolError,
    AcceptError(std::io::Error)
}

pub struct GameServer {
    addr: SocketAddr,
    lobby: Lobby,
    backend: BackendConnection,
}

pub struct GameClient {
    addr: SocketAddr,
    client: Client<TcpStream>,
}

impl GameClient {
    fn from_raw(client: Client<TcpStream>) -> Result<Self, ()> {
        let addr = client.peer_addr().map_err(|_| ())?;
        info!("got a client connection from: {}", addr);
        Ok(GameClient {
            addr,
            client,
        })
    }

    fn require_token(&mut self) -> Option<Token> {
        let message = self.client
                 .recv_message()
                 .ok()?;
        if let OwnedMessage::Text(text) = message {
            text.parse().ok()
        } else {
            None
        }
    }

    fn host_name(&self) -> SocketAddr {
        self.addr
    }

    pub fn split(self) -> (ClientSender, ClientReceiver) {
        let (mut rec, mut sen) = self.client.split().unwrap();
        (sen, rec)
    }
}

type ClientConnection = Result<GameClient, GameServerError>;

impl GameServer {
    pub fn new<T: ToSocketAddrs>(addr: T) -> Self {
        let addr = addr.to_socket_addrs().unwrap().next().unwrap();
        debug!("ws address: {}", addr);
        info!("create lobby");
        let lobby = Lobby::new();
        let backend = BackendConnection::new("https://kobert.dev");
        info!("got a C# backend connection");
        GameServer {
            addr,
            lobby: lobby,
            backend: backend,
        }
    }

    pub fn run(&mut self) -> Result<(), GameServerError> {
        let reader = self.read_clients();
        loop {
            let connection = reader.recv().unwrap()?;
            self.add_client(connection);
        }
    }

    fn add_client(&mut self, mut client: GameClient) {
        let token = client.require_token();
        if let Some(token) = token {
            let result = self.backend.validate_token(&token);
            match result {
                Err(err) => warn!("client's token {} is not valid: '{:?}'",
                                  token, err),
                Ok(result) => {
                    debug!("client validation was successfull");
                    let user_id = result.user_id;
                    let group_id = result.group_id;
                    let group_type = result.group_type;
                    let group_name = result.group_name;
                    debug!("add client: (id:{}, token:{}, host:{}) to \"{}\"",
                        user_id, token, client.host_name(), group_name);
                    //clients.lock().unwrap().insert(token, client);
                    self.lobby.add_client(&group_type, group_id,
                                     &group_name, user_id, client);
                }
            }
        } else {
            warn!("client sent invalid token");
        }
    }

    fn read_clients(&self) -> Receiver<ClientConnection> {
        let (s, r): (Sender<ClientConnection>, Receiver<ClientConnection>)
                     = mpsc::channel();
        let addr = self.addr;
        std::thread::spawn(move || {
            let result = Self::handle_requests(addr, &s).or_else(|e| s.send(Err(e)));
        });
        r
    }

    fn handle_requests(addr: SocketAddr, s: &Sender<ClientConnection>) -> Result<(), GameServerError> {
        let server = match Server::<NoTlsAcceptor>::bind(addr) {
            Ok(v) => v,
            Err(e) => {
                error!("websocket binding error");
                Err(GameServerError::BindError(e))?
            },
        };
        info!("webserver is being launched");
        for req in server {
            s.send(Ok(Self::handle_request(req)?)).unwrap();
        }
        info!("webserver is being shut down");
        Ok(())
    }

    fn handle_request(req: AcceptResult<TcpStream>) -> ClientConnection {
        match req {
            Ok(req) => {
                if !req.protocols().contains(&PROTOCOL.to_string()) {
                    warn!("a client tried to connect without {} protocol", PROTOCOL);
                    req.reject().unwrap();
                    Err(GameServerError::InvalidProtocolError)
                } else {
                    match req.use_protocol(PROTOCOL).accept() {
                        Ok(client) => {
                            match GameClient::from_raw(client) {
                                Ok(client) => Ok(client),
                                Err(_) => {
                                    error!("could not create a client");
                                    Err(GameServerError::HandshakeRequestError)
                                }
                            }
                        },
                        Err((_, e)) => {
                            warn!("client handshake failed");
                            Err(GameServerError::AcceptError(e))
                        }
                    }
                }
            },
            Err(_) => {
                warn!("invalid client request");
                Err(GameServerError::HandshakeRequestError)
            }
        }
    }
}
