use anyhow::Result;

use tokio::io::{AsyncRead, AsyncWrite, BufReader,
    AsyncReadExt, AsyncBufReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::{mpsc, oneshot};

use crate::message;
use crate::message::Message;
use crate::serializable::Serializable;

type MessageReturnError = (Message, oneshot::Sender<Result<()>>);

#[derive(Debug)]
pub struct MessageQueue {
    recv_loop_receiver: mpsc::UnboundedReceiver<Result<Message>>,
    send_loop_sender: mpsc::UnboundedSender<MessageReturnError>,
}

impl MessageQueue {
    pub fn new<T>(stream: T) -> Self where
        T: 'static + AsyncRead + AsyncWrite + Send {
        let (read_stream, write_stream) = tokio::io::split(stream);

        let (recv_loop_sender, recv_loop_receiver)
            = mpsc::unbounded_channel::<Result<Message>>();
        let (send_loop_sender, send_loop_receiver)
            = mpsc::unbounded_channel::<MessageReturnError>();

        tokio::task::spawn(async move {
            recv_loop(read_stream, recv_loop_sender).await;
        });

        tokio::spawn(async move {
            send_loop(write_stream, send_loop_receiver).await;
        });

        Self { recv_loop_receiver, send_loop_sender }
    }

    pub async fn recv(&mut self) -> Option<Result<Message>> {
        recv_priv(&mut self.recv_loop_receiver).await
    }

    pub async fn send(&self, message: Message) -> Result<()> {
        send_priv(&self.send_loop_sender, message).await
    }

    pub fn split(self) -> (RecvHalf, SendHalf) {
        (
            RecvHalf { recv_loop_receiver: self.recv_loop_receiver },
            SendHalf { send_loop_sender: self.send_loop_sender }
        )
    }
}

async fn recv_priv(recv_loop_receiver: &mut mpsc::UnboundedReceiver<Result<Message>>)
    -> Option<Result<Message>> {
    recv_loop_receiver.recv().await
}

async fn send_priv(
    send_loop_sender: &mpsc::UnboundedSender<MessageReturnError>,
    message: Message,
) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    let full = (message, tx);
    send_loop_sender.send(full)?;
    if let Err(e) = rx.await {
        return Err(e.into());
    }
    Ok(())
}

/// reads messages from `stream`, deserializes them, and sends them to `queue`
async fn recv_loop<T>(
    stream: ReadHalf<T>,
    recv_loop_sender: mpsc::UnboundedSender<Result<Message>>,
) where T: AsyncRead {
    let mut stream = BufReader::new(stream);
    loop {
        if let Err(_) = recv_loop_sender.send(recv_loop_inner(&mut stream).await) {
            // nothing can be done here
            return;
        }
    }
}

async fn recv_loop_inner<T>(stream: &mut BufReader<T>) -> Result<Message> where
    T: AsyncRead + Unpin {
    // read until message start byte
    sink_until(stream, message::MESSAGE_START).await?;

    let mut buf = vec![message::MESSAGE_START];
    stream.read_until(message::MESSAGE_END, &mut buf).await?;

    let message = Message::deserialize(&buf)?;
    println!("recv: {:?}", message);

    Ok(message)
}

async fn sink_until<T>(stream: &mut BufReader<T>, byte: u8) -> Result<()> where
    T: AsyncRead + Unpin {
    loop {
        let b = stream.read_u8().await?;
        if b == byte {
            break;
        }
    }
    Ok(())
}

/// receives messages from `queue`, serializes them, and writes them to `stream`
async fn send_loop<T>(
    mut stream: WriteHalf<T>,
    mut send_loop_receiver: mpsc::UnboundedReceiver<MessageReturnError>
) where T: AsyncWrite {
    loop {
        let (message, tx) = match send_loop_receiver.recv().await {
            Some(x) => x,
            None => break,
        };
        if let Err(_) = tx.send(send_loop_inner(&mut stream, message).await) {
            // nothing can be done here
            return;
        }
    }
}

async fn send_loop_inner<T>(stream: &mut WriteHalf<T>, message: Message) -> Result<()> where
    T: AsyncWrite {
    println!("send: {:?}", message);
    stream.write_all(&message.serialize()).await.map_err(|e| e.into())
}

pub struct RecvHalf {
    recv_loop_receiver: mpsc::UnboundedReceiver<Result<Message>>,
}

impl RecvHalf {
    pub async fn recv(&mut self) -> Option<Result<Message>> {
        recv_priv(&mut self.recv_loop_receiver).await
    }

    pub fn unsplit(self, send_half: SendHalf) -> MessageQueue {
        MessageQueue {
            recv_loop_receiver: self.recv_loop_receiver,
            send_loop_sender: send_half.send_loop_sender,
        }
    }
}

pub struct SendHalf {
    send_loop_sender: mpsc::UnboundedSender<MessageReturnError>,
}

impl SendHalf {
    pub async fn send(&mut self, message: Message) -> Result<()> {
        send_priv(&self.send_loop_sender, message).await
    }
}
