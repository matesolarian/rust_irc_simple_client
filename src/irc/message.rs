//! The message module contains the `Message` struct which represents an
//! IRC message either being received from the server or sent by the client.
//!
//! The module also contains several constructor methods for constructing
//! messages to be sent to the server.


use irc::command::{Command, ArgumentIter};
use irc::error::{Result, Error};
use std::ops::Range;

use irc::parser;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PrefixRange {
    pub raw_prefix: Range<usize>,
    pub prefix: Range<usize>,
    pub user: Option<Range<usize>>,
    pub host: Option<Range<usize>>,
}

pub type TagRange = (Range<usize>, Option<Range<usize>>);

/// Representation of IRC messages that splits a message into its constituent
/// parts specified in RFC1459 and the IRCv3 spec.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Message {
    pub message: String,
    pub tags: Option<Vec<TagRange>>,
    pub prefix: Option<PrefixRange>,
    pub command: Range<usize>,
    pub arguments: Option<Vec<Range<usize>>>,
}

impl Message {
    /// Attempt to construct a new message from the given raw IRC message.
    pub fn try_from(value: String) -> Result<Message> {
        let result = parser::parse_message(value)?;

        Ok(result)
    }

    /// A strongly typed interface for determining the type of the command
    /// and retrieving the values of the command.
    pub fn command<'a, T>(&'a self) -> Option<T>
    where
        T: Command<'a>,
    {
        <T as Command>::try_match(self.raw_command(), self.raw_args())
    }

    /// Retrieves the prefix for this message, if there is one.  If there is either
    /// a user or host associated with the prefix, it will also return those.
    pub fn prefix(&self) -> Option<(&str, Option<&str>, Option<&str>)> {
        if let Some(ref prefix_range) = self.prefix {
            let user = prefix_range.user.clone().map(|user| &self.message[user]);
            let host = prefix_range.host.clone().map(|host| &self.message[host]);

            Some((&self.message[prefix_range.prefix.clone()], user, host))
        } else {
            None
        }
    }

    /// Retrieve the raw command associated with this message.
    pub fn raw_command(&self) -> &str {
        &self.message[self.command.clone()]
    }

    /// Get an iterator to the raw arguments associated with this message.
    pub fn raw_args(&self) -> ArgumentIter {
        if let Some(ref arguments) = self.arguments {
            ArgumentIter::new(&self.message, arguments.iter())
        } else {
            ArgumentIter::new(&self.message, [].iter())
        }
    }

    /// Get the raw IRC command this message was constrcuted from.
    pub fn raw_message(&self) -> &str {
        &self.message
    }
    
    /// Constructs a message containing a PONG command targeting the specified host.
    pub fn pong(host: &str) -> Result<Message> {
        Message::try_from(format!("PONG {}", host))
    }
    
    /// Constructs a message containing a NICK command with the specified nickname.
    pub fn nick(nick: &str) -> Result<Message> {
        Message::try_from(format!("NICK {}", nick))
    }
    
    /// Constructs a message containing a USER command with the specified username and real name.
    pub fn user(username: &str, real_name: &str) -> Result<Message> {
        Message::try_from(format!("USER {} 0 * :{}", username, real_name))
    }
    
    /// Constructs a message containing a JOIN command for the specified channel.
    /// The `channels` parameter is a comma separated list of channels to join.
    /// The `keys` parameter is an optional comma separated list of passwords for the channels being joined.
    pub fn join(channels: &str, keys: Option<&str>) -> Result<Message> {
        let command = if let Some(keys) = keys {
            format!("JOIN {} {}", channels, keys)
        } else {
            format!("JOIN {}", channels)
        };
    
        Message::try_from(command)
    }
    
    /// Constructs a message containing a PRIVMSG command sent to the specified targets with the given message.
    pub fn priv_msg(targets: &str, message: &str) -> Result<Message> {
        Message::try_from(format!("PRIVMSG {} :{}", targets, message))
    }

}

impl ::std::str::FromStr for Message {
    type Err = Error;

    fn from_str(input: &str) -> Result<Message> {
        Message::try_from(input.to_owned())
    }
}
