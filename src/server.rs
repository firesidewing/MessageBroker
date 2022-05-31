use std::{
    collections::HashMap,
    io::ErrorKind,
    mem::MaybeUninit,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use bytes::{BufMut, BytesMut};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener, TcpStream,
    },
    sync::watch::{self, Receiver, Sender},
};
use tracing::{debug, error, info, info_span, Instrument};

use crate::server::message::MessageType;

mod message;

pub const INPUT_BUFFER_SIZE: usize = u16::MAX as usize; // 2^16 - 1

pub type SharedClients = Arc<Mutex<HashMap<SocketAddr, String>>>;
pub type MessageMap = HashMap<String, Subscription<Vec<u8>>>;
pub type SharedMessageMap = Arc<Mutex<MessageMap>>;

#[derive(Debug)]
pub struct Subscription<T> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}

#[derive(Debug, Clone)]
pub struct State {
    clients: Arc<Mutex<HashMap<SocketAddr, String>>>,
}

#[tracing::instrument]
pub async fn run_server() -> Result<()> {
    let addr: SocketAddr = "0.0.0.0:6969".parse()?;
    info!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let clients: SharedClients = Arc::new(Mutex::new(HashMap::new()));

    let messages: Arc<Mutex<MessageMap>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().instrument(info_span!("accept")).await?;
        let clients = clients.clone();
        let messages = messages.clone();

        tokio::spawn(async move {
            match handle_connection(stream, addr, clients, messages).await {
                Err(err) => {
                    error!(%err, "Error handling connection" );
                }
                _ => (),
            }
        });
    }
}

#[tracing::instrument(skip(stream, clients, messages))]
async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    clients: SharedClients,
    messages: SharedMessageMap,
) -> Result<()> {
    debug!("Got connection from {}", addr);

    let mut message_type = MessageType::None;
    let mut bytes_to_read = 0;
    let mut type_length = 0;

    loop {
        let buffer: MaybeUninit<[u8; INPUT_BUFFER_SIZE]> = MaybeUninit::uninit();
        let mut input_buffer = unsafe { buffer.assume_init() }; // Avoid to initialize the array

        match stream.read(&mut input_buffer).await {
            Ok(0) => {
                remove_client(&clients, &addr)?;
                break;
            }
            Ok(size) => {
                if size < 5 {
                    continue;
                }

                info!("Got message of size {}", size);

                let message = &input_buffer[..size];

                if let MessageType::None = message_type {
                    message_type = MessageType::parse_from_header(&message[0]);
                    type_length = i32::from_le_bytes(message[1..5].try_into().unwrap());

                    if let MessageType::Name = message_type {
                        add_client(&clients, &addr, name);
                        continue;
                    }

                    process_message(&message_type, &message[5..], &addr, &state)?;

                    continue;
                }

                process_message(&message_type, &message, &addr, &state)?;
            }
            Err(ref err) if err.kind() == ErrorKind::ConnectionReset => {
                remove_client(&clients, &addr)?;
                break;
            }
            Err(err) => {
                debug!("TCP receive error: {}", err);
                break;
            }
        }
    }

    Ok(())
}

fn process_message(message_type: &MessageType, buffer: &[u8], addr: &SocketAddr) -> Result<()> {
    match message_type {
        MessageType::None => Ok(()),
        MessageType::Name => MessageType::parse_name(&buffer, &addr),
        MessageType::Message => MessageType::send_to_subscribed(&buffer),
        MessageType::Subscribe => MessageType::subscribe(&buffer),
        MessageType::Unsubscribe => todo!(),
        MessageType::Broadcast => todo!(),
    }
}

#[tracing::instrument(skip(clients))]
fn add_client(clients: &SharedClients, addr: &SocketAddr, name: String) -> Result<()> {
    let mut clients = clients.lock().unwrap();
    clients.insert(addr.clone(), name.to_string());

    info!("Client added {}", addr);

    Ok(())
}

#[tracing::instrument(skip(clients))]
fn remove_client(clients: &SharedClients, addr: &SocketAddr) -> Result<()> {
    let mut clients = clients.lock().unwrap();
    clients.remove(addr).context("Client not found")?;
    info!("Client removed {}", addr);

    Ok(())
}

#[tracing::instrument(skip(stream))]
async fn return_error(err: anyhow::Error, stream: &mut TcpStream) -> Result<()> {
    let err_str = err.to_string();
    let mut buf = BytesMut::with_capacity(err_str.len() + 1);
    buf.put_u8(0);
    buf.put(err_str.as_bytes());

    stream.write(&buf).await?;

    Ok(())
}
