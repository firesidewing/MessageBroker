use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use anyhow::{Error, Result};
use thiserror::Error;

use tokio::net::TcpStream;
use tracing::info;

use crate::server::SharedState;

#[derive(Debug)]
pub enum MessageType {
    None,
    Name,
    Message,
    Subscribe,
    Unsubscribe,
    Broadcast,
}

impl MessageType {
    pub fn parse_from_header(header: &u8) -> MessageType {
        match header {
            1 => MessageType::Name,
            2 => MessageType::Message,
            3 => MessageType::Subscribe,
            4 => MessageType::Unsubscribe,
            5 => MessageType::Broadcast,
            _ => MessageType::None,
        }
    }

    #[tracing::instrument(skip(addr, state, input))]
    pub fn parse_name(
        input: &[u8],
        addr: &SocketAddr,
        state: &SharedState,
    ) -> Result<&str, MessageError> {
        match std::str::from_utf8(input) {
            Ok(name) => name,
            Err(_) => Err(MessageError::InvalidName.into()),
        }
    }

    #[tracing::instrument(skip(state, input))]
    pub fn send_to_subscribed(input: &[u8], state: &SharedState) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(input, stream))]
    pub fn subscribe(input: &[u8], stream: &SharedState) -> Result<()> {
        // let messages = messages.lock().unwrap();
        // let name = match std::str::from_utf8(input) {
        //     Ok(name) => name,
        //     Err(_) => return Err(MessageError::InvalidName.into()),
        // };

        // if messages.contains_key(name) {
        //     info!("Subscribing {} to {}", stream.peer_addr()?, name);
        //     messages.get_mut(name).unwrap().push(stream.clone());
        // } else {
        //     info!("Creating new channel {}", name);
        //     let mut messages = messages.clone();
        //     messages.insert(name.to_string(), vec![stream.clone()]);
        // }

        Ok(())
    }

    #[tracing::instrument(skip(input, state))]
    pub fn unsubscribe(input: &[u8], state: &SharedState) -> Result<()> {
        let state = state.lock().unwrap();

        Ok(())
    }

    #[tracing::instrument(skip(input, state))]
    pub fn broadcast(input: &[u8], state: &SharedState) -> Result<()> {
        let state = state.lock().unwrap();

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("Un Supported Message Type, found {header:?}")]
    InvalidMessageType { header: u8 },
    #[error("Invalid Message")]
    InvalidMessage,
    #[error("Invalid name, must be UTF-8")]
    InvalidName,
    #[error("Invalid subscribe type, must be UTF-8")]
    InvalidSubscribe,
    #[error("Invalid subscribe type, must be UTF-8")]
    InvalidUnsubscribe,
    #[error("Invalid broadcast name, must be UTF-8")]
    InvalidBroadcast,
}
