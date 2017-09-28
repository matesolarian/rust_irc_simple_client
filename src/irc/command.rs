//! The command module contains everything needed to perform strongly typed access
//! to commands associated with a message.

use std::ops::Range;
use std::slice::Iter;

use irc::command;

/// An implementation of Iterator that iterates over the arguments of a `Message`.
#[derive(Clone)]
pub struct ArgumentIter<'a> {
    source: &'a str,
    iter: Iter<'a, Range<usize>>,
}

impl<'a> ArgumentIter<'a> {
    // This is intended for internal usage and thus hidden.
    #[doc(hidden)]
    pub fn new(source: &'a str, iter: Iter<'a, Range<usize>>) -> ArgumentIter<'a> {
        ArgumentIter {
            source: source,
            iter: iter,
        }
    }
}

impl<'a> Iterator for ArgumentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|range| &self.source[range.clone()])
    }
}

impl<'a> DoubleEndedIterator for ArgumentIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|range| &self.source[range.clone()])
    }
}

/// The `Command` trait is a trait that's implemented by types wishing to provide command
/// parsing capability for usage with the `Message::command` method.
pub trait Command<'a> {
    /// Provides the name of the command to be matched. Examples include `PRIVMSG` or `PING`.
    fn name() -> &'static str;

    /// This method takes in an iterator of arguments associated with a `Message` and attempts
    /// to parse the arguments into a matched `Command`.  If no match is found, None is returned.
    fn parse(arguments: ArgumentIter<'a>) -> Option<Self>
    where
        Self: Sized;

    /// A default implementation that takes in the given command name and arguments and attempts to match
    /// the command and parse the arguments into a strongly typed representation. If there is no match
    /// or the parse fails, it returns `None`.
    fn try_match(command: &str, arguments: ArgumentIter<'a>) -> Option<Self>
    where
        Self: Sized,
    {
        if command == Self::name() {
            Self::parse(arguments)
        } else {
            None
        }
    }
}

/// A macro for simplifying the process of matching commands.
#[macro_export]
macro_rules! command_match {
    (@message=$message:expr => $command:pat => $body:expr) => {{
        let $command = $message;
        $body
    }};

    (@message=$message:expr => $command:pat => $body:expr, $($rest:tt)*) => {
        match $message.command() {
            Some($command) => $body,
            _ => command_match!(@message=$message => $($rest)*)
        }
    };

    ($message:expr => { $($rest:tt)* }) => {{
        let message = $message;
        command_match!(@message=message => $($rest)*)
    }};
}

/// A macro for creating implementations of basic commands with up to four
/// &str arguments.
///

#[macro_export]
macro_rules! command {
    ($(#[$meta:meta])* ($command:expr => $command_name:ident())) => {
        $(#[$meta])*
        pub struct $command_name;

        impl<'a> $crate::command::Command<'a> for $command_name {
            fn name() -> &'static str {
                $command
            }

            fn parse(_: $crate::command::ArgumentIter<'a>) -> Option<$command_name> {
                Some($command_name)
            }
        }
    };

    ($(#[$meta:meta])* ($command:expr => $command_name:ident($($name:ident),+))) => {
        $(#[$meta])*

        pub struct $command_name<'a>($(pub expand_param!($name)),+);

        impl<'a> command::Command<'a> for $command_name<'a> {
            fn name() -> &'static str {
                $command
            }

            fn parse(mut arguments: command::ArgumentIter<'a>) -> Option<$command_name> {
                $(
                    let $name = match arguments.next() {
                        Some(value) => value,
                        None => return None
                    };
                )+

                Some($command_name($($name),*))
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_param {
    ($i:ident) =>  { &'a str };
}

command! { 
    /// Represents a PING command.  The first element is the host.
    ("PING" => Ping(host)) 
}

command! {
    /// Represents a PONG command. The first element is the host.
    ("PONG" => Pong(host))
}

command! {
    /// Represents a PRIVMSG command.  The first element is the target of the message and
    ("PRIVMSG" => PrivMsg(target, message))
}

// command! {
//     ("JOIN" => Join(channel))
// }

// command! {
//     ("PART" => Part(channel))
// }

command! { 
    /// Represents a WELCOME numeric. The first element is the unsername and the second element is the welcome message.
    ("001" => Welcome(user, message))
}

command! {
    /// Represents a YOURHOST numeric. The first element is the unsername and the second element is the yourhost message.
    ("002" => YourHost(user, message))
}

command!{
    /// Represents a CREATED numeric. The first element is the unsername and the second element is the created message.
    ("003" => Created(user, message))
}

command!{
    /// Represents a MYINFO numeric. The first element is the username and the second element is the server info message.
    ("004" => ServerInfo(user, message))
}

#[derive(PartialEq, Debug)]
pub enum NamesReplyChannelType {
    Secret,
    Private,
    Other,
}

pub struct NamesReply<'a>(pub NamesReplyChannelType, pub &'a str, pub Vec<&'a str>);

impl<'a> Command<'a> for NamesReply<'a> {
    fn name() -> &'static str {
        "353"
    }

    fn parse(arguments: ArgumentIter<'a>) -> Option<NamesReply<'a>> {
        // NOTE: Since the first parameter is optional, it's just easier to extract
        // components in reverse.
        let mut arguments = arguments.rev();

        let names = match arguments.next() {
            Some(names) => names.split_whitespace(),
            None => return None,
        };

        let channel = match arguments.next() {
            Some(channel) => channel,
            None => return None,
        };

        let channel_type = match arguments.next() {
            Some(channel_type) => {
                match channel_type {
                    "@" => NamesReplyChannelType::Secret,
                    "*" => NamesReplyChannelType::Private,
                    _ => NamesReplyChannelType::Other,
                }
            }
            None => NamesReplyChannelType::Other,
        };

        Some(NamesReply(channel_type, channel, names.collect()))
    }
}

pub struct EndNamesReply<'a>(pub &'a str, pub &'a str);

impl<'a> Command<'a> for EndNamesReply<'a> {
    fn name() -> &'static str {
        "366"
    }

    fn parse(arguments: ArgumentIter<'a>) -> Option<EndNamesReply<'a>> {
        // NOTE: Some servers are bad and include non-standard args at the start.
        // So the parameters are extracted in reverse to compensate.
        let mut arguments = arguments.rev();

        let message = match arguments.next() {
            Some(message) => message,
            None => return None,
        };

        let channel = match arguments.next() {
            Some(channel) => channel,
            None => return None,
        };

        Some(EndNamesReply(channel, message))
    }
}

