
extern crate tokio_io;
extern crate tokio_core;
extern crate futures;

use futures::{Sink, Stream, Poll, StartSend, Async};
use self::tokio_io::{AsyncRead, AsyncWrite};
use self::tokio_io::codec::Framed;

use irc::codec;
use irc::message::{Message};
use irc::error::{Error, ErrorKind};

use std;
use std::time;
use std::io::Write;

const PING_TIMEOUT_IN_SECONDS: u64 = 10 * 60;


/// `IrcTransport` represents a framed IRC stream returned from the connection
/// methods when their given futures are resolved. It internally handles the
/// processing of PING requests and timing out the connection when no PINGs
/// have been recently received from the server.
///
/// It is possible to split `IrcTransport` into `Stream` and `Sink` via the
/// the `split` method.
pub struct IrcTransport<T>
    where T: AsyncRead + AsyncWrite
{
    pub inner: Framed<T, codec::IrcCodec>,
    last_ping: time::Instant,
}

impl<T> IrcTransport<T>
    where T: AsyncRead + AsyncWrite
{
    pub fn new(inner: Framed<T, codec::IrcCodec>) -> IrcTransport<T> {
        IrcTransport {
            inner: inner,
            last_ping: time::Instant::now(),
        }
    }
}

impl<T> Stream for IrcTransport<T>
    where T: AsyncRead + AsyncWrite
{
    type Item = Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.last_ping.elapsed().as_secs() >= PING_TIMEOUT_IN_SECONDS {
            self.close()?;
            return Err(ErrorKind::ConnectionReset.into());
        }

        loop {
            match try_ready!(self.inner.poll()) {
                Some(ref message) if message.raw_command() == "PING" => {
                    self.last_ping = time::Instant::now();

                    if let Some(host) = message.raw_args().next() {
                        let result = self.inner.start_send(Message::pong(host)?)?;

                        assert!(result.is_ready());

                        self.inner.poll_complete()?;
                    }
                }
                message => return Ok(Async::Ready(message)),
            }
        }
    }
}

impl<T> Sink for IrcTransport<T>
    where T: AsyncRead + AsyncWrite
{
    type SinkItem = Message;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        Ok(self.inner.start_send(item)?)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(self.inner.poll_complete()?)
    }
}


