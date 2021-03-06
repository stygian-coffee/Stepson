pub mod completion;
pub mod from_repl;

pub use completion::*;
pub use from_repl::*;

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use rustyline::config::{CompletionType, Config};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::bluetooth::{AsyncBtStream, Device, Manager};
use crate::message::Message;
use crate::message_queue::MessageQueue;

type ShouldExit = bool;

struct ReplData {
    manager: Manager,
    device: Option<Device>,
    message_queue: Option<MessageQueue>,
}

pub struct Repl {
    data: Rc<RefCell<ReplData>>,
}

impl ReplCompletionStateful for ReplData {
    fn completion_tree(&self, cx: std::rc::Rc<CompletionContext>) -> CompletionTree {
        CompletionTree::new(vec![
            ("connect".to_string(), CompletionTree::lazy_empty()),
            ("devices".to_string(), CompletionTree::lazy_empty()),
            ("sendll".to_string(), Message::lazy_completion_tree(cx)),
            ("sendll".to_string(), CompletionTree::lazy_empty()),
            ("quit".to_string(), CompletionTree::lazy_empty()),
        ])
    }
}

impl Repl {
    pub fn new() -> Result<Self> {
        let data = Rc::new(RefCell::new(ReplData {
            manager: Manager::new()?,
            device: None,
            message_queue: None,
        }));
        Ok(Self { data })
    }

    pub async fn run(&mut self) -> Result<()> {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::<ReplHelper>::with_config(config);
        rl.set_helper(Some(ReplHelper {
            data: self.data.clone(),
        }));

        loop {
            let readline = rl.readline(&self.prompt());
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    if self.execute_command(line).await {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {}
                Err(ReadlineError::Eof) => break,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }

    fn prompt(&self) -> String {
        let data = self.data.borrow();
        match &data.device {
            Some(dev) => format!("[{}]# ", dev.name),
            None => "[None]# ".to_string(),
        }
    }

    // return true means exit the repl
    async fn execute_command(&mut self, command: String) -> ShouldExit {
        let mut words = command.split_whitespace();
        let res = match words.next() {
            None => Ok(false),
            Some("connect") => self.connect(&mut words).await,
            Some("devices") => self.devices(&mut words).await,
            Some("send") => self.send(&mut words).await,
            Some("quit") => self.quit(&mut words).await,
            Some(w) => self.unknown_command(w),
        };
        match res {
            Ok(b) => b,
            Err(e) => {
                println!("{}", e);
                false
            }
        }
    }

    fn unknown_command(&self, word: &str) -> Result<ShouldExit> {
        println!("unknown command: {}", word);
        Ok(false)
    }

    async fn connect<'a, T>(&mut self, words: &mut T) -> Result<ShouldExit>
    where
        T: Iterator<Item = &'a str>,
    {
        let addr = match words.next() {
            Some(w) => w.to_uppercase(),
            None => {
                println!("connect: too few arguments, expected 1");
                return Ok(false);
            }
        };

        if let Some(_) = words.next() {
            println!("connect: too many arguments, expected 1");
            return Ok(false);
        }

        let devices = self.data.borrow().manager.get_devices()?;
        let device = match devices.into_iter().find(|dev| dev.addr == addr) {
            Some(dev) => dev,
            None => {
                println!("connect: no device found with MAC address {}", addr);
                return Ok(false);
            }
        };

        let bt_stream = AsyncBtStream::new(device.bt_stream()?)?;
        let message_queue = MessageQueue::new(bt_stream);

        println!("connect: connected to {}", device.name);

        self.data.borrow_mut().device = Some(device);
        self.data.borrow_mut().message_queue = Some(message_queue);

        Ok(false)
    }

    async fn devices<'a, T>(&self, words: &mut T) -> Result<ShouldExit>
    where
        T: Iterator<Item = &'a str>,
    {
        if let Some(_) = words.next() {
            println!("devices: too many arguments, expected 0");
            return Ok(false);
        }

        let devices = self.data.borrow().manager.get_devices()?;

        for dev in devices {
            println!("{}", dev);
        }

        Ok(false)
    }

    async fn send<'a, T>(&self, words: &mut T) -> Result<ShouldExit>
    where
        T: Iterator<Item = &'a str>,
    {
        let data = self.data.borrow();
        let message_queue = match &data.message_queue {
            Some(s) => s,
            None => {
                println!("send: not connected to a device");
                return Ok(false);
            }
        };

        let message = match Message::from_repl(words) {
            Ok(m) => m,
            Err(e) => {
                println!("send: {}", e);
                return Ok(false);
            }
        };

        if let Err(e) = message_queue.send(message).await {
            println!("send: unable to send message: {}", e);
        }

        Ok(false)
    }

    async fn quit<'a, T>(&mut self, words: &mut T) -> Result<ShouldExit>
    where
        T: Iterator<Item = &'a str>,
    {
        match words.next() {
            Some(_) => {
                println!("quit: too many arguments, expected 0");
                Ok(false)
            }
            None => Ok(true),
        }
    }
}
