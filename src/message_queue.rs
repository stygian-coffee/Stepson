use tokio::io::{AsyncRead, AsyncWrite, BufReader,
    AsyncReadExt, AsyncBufReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::{mpsc, oneshot};

use crate::message;
use crate::message::Message;
use crate::serializable::Serializable;

#[derive(Debug)]
struct SendCommand {
    message: Message,
    resp: oneshot::Sender<()>,
}

pub struct MessageQueue {
    recv_loop_receiver: mpsc::UnboundedReceiver<SendCommand>,
    send_loop_sender: mpsc::UnboundedSender<SendCommand>,
}

impl MessageQueue {
    pub fn new<T>(stream: T) -> Self where
        T: 'static + AsyncRead + AsyncWrite + Send {
        let (read_stream, write_stream) = tokio::io::split(stream);

        let (recv_loop_sender, recv_loop_receiver) = mpsc::unbounded_channel::<SendCommand>();
        let (send_loop_sender, send_loop_receiver) = mpsc::unbounded_channel::<SendCommand>();

        let clone = send_loop_sender.clone();
        tokio::task::spawn(async move {
            Self::recv_loop(read_stream, clone, recv_loop_sender).await;
        });

        tokio::spawn(async move {
            Self::send_loop(write_stream, send_loop_receiver).await;
        });

        Self { recv_loop_receiver, send_loop_sender }
    }

    /// reads messages from `stream`, deserializes them, and sends them to `queue`
    async fn recv_loop<T>(
        stream: ReadHalf<T>,
        send_loop_sender: mpsc::UnboundedSender<SendCommand>,
        recv_loop_sender: mpsc::UnboundedSender<SendCommand>,
    ) where T: AsyncRead {
        let mut stream = BufReader::new(stream);
        loop {
            // read until message start byte
            loop {
                let b = stream.read_u8().await.unwrap(); //TODO
                if b == message::MESSAGE_START {
                    break;
                }
            }
            let mut buf = vec![message::MESSAGE_START];
            stream.read_until(message::MESSAGE_END, &mut buf).await; //TODO

            let deserialized = Message::deserialize(&buf).unwrap(); //TODO
            println!("recv: {:?}", deserialized);

            if deserialized.requires_ack() {
                let (tx, rx) = oneshot::channel();
                send_loop_sender.send(SendCommand {
                    message: Message {
                        sequence_number: 1,
                        data: message::Data::Ack(message::ack::Ack {}),
                    },
                    resp: tx,
                }); //TODO
                rx.await; //TODO
            }

            let (tx, rx) = oneshot::channel();
            let command = SendCommand {
                message: deserialized,
                resp: tx,
            };
            recv_loop_sender.send(command); //TODO
            rx.await; //TODO
        }
    }

    /// receives messages from `queue`, serializes them, and writes them to `stream`
    async fn send_loop<T>(
        mut stream: WriteHalf<T>,
        mut send_loop_receiver: mpsc::UnboundedReceiver<SendCommand>
    ) where T: AsyncWrite {
        loop {
            let command = send_loop_receiver.recv().await.unwrap(); //TODO
            stream.write_all(&command.message.serialize()).await; //TODO
            println!("send: {:?}", command.message);
            command.resp.send(()); //TODO
        }
    }

    pub async fn recv(&mut self) -> Message {
        self.recv_loop_receiver.recv().await.unwrap().message //TODO
    }

    pub async fn send(&mut self, message: Message) {
        let (tx, rx) = oneshot::channel();
        let command = SendCommand {
            message,
            resp: tx,
        };
        self.send_loop_sender.send(command).unwrap(); //TODO
        rx.await; //TODO
    }
}
