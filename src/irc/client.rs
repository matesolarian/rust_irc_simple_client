
//! The client module contains all types needed to make a connection
//! to a remote IRC host.
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;



use futures::{Future, Async, Poll};

use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};
use self::tokio_io::{AsyncRead};

use std::net::SocketAddr;
//use std::time;

use irc::transport::{IrcTransport};
use irc::codec;
use irc::error::{Error};

/// A light-weight client type for establishing connections to remote servers.
/// This type consumes a given `SocketAddr` and provides several methods for
/// establishing connections to a remote server.  
/// Currently these methods allow for the connection to a server with unencrypted data.
/// Each of the connection methods will return a future, that when successfully
/// resolved, will provide a `Stream` that allows for communication with the
/// remote server.
pub struct Client {
    host: SocketAddr,
}


impl Client {
    /// Create a new instance of `Client` that will connect to host
    pub fn new<H: Into<SocketAddr>>(host: H) -> Client {
        Client { host: host.into() }
    }
    
    /// Returns a future, that when resolved provides an unecrypted `Stream`
    /// that can be used to receive `Message` from the server and send `Message`
    /// to the server.
    ///
    /// The resulting `Stream` can be `split` into a separate `Stream` for
    /// receiving `Message` from the server and a `Sink` for sending `Message`
    /// to the server.
    pub fn connect(&self, handle: &Handle) -> ClientConnectFuture {
        let tcp_stream = TcpStream::connect(&self.host, handle);

        ClientConnectFuture { inner: tcp_stream }
    }
}

/// Represents a future, that when resolved provides an unecrypted `Stream`
/// that can be used to receive `Message` from the server and send `Message`
/// to the server.
pub struct ClientConnectFuture {
    inner: TcpStreamNew,
}

impl Future for ClientConnectFuture {
    type Item = IrcTransport<TcpStream>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let framed = try_ready!(self.inner.poll()).framed(codec::IrcCodec);
        let irc_transport = IrcTransport::new(framed);

        Ok(Async::Ready(irc_transport))
    }
}




