pub mod completion;
pub mod from_repl;

pub use completion::*;
pub use from_repl::*;

use std::collections::HashMap;

use anyhow::Result;
use rustyline::config::{CompletionType, Config};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::bluetooth::{AsyncBtStream, Device, Manager};
use crate::message::Message;
use crate::message_queue::MessageQueue;

type ShouldExit = bool;

pub struct Repl {
    manager: Manager,
    device: Option<Device>,
    message_queue: Option<MessageQueue>,
}

impl ReplCompletion for Repl {
    fn possible_completions<'a>() -> CompletionMap<'a> {
        let mut completions = HashMap::new();
        completions.insert("connect", None);
        completions.insert("devices", None);
        completions.insert("send", None);
        completions.insert("quit", None);
        completions.into()
    }
}

impl Repl {
    pub fn new() -> Result<Self> {
        Ok(Self { 
            manager: Manager::new()?,
            device: None,
            message_queue: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::<ReplHelper<'_>>::with_config(config);
        rl.set_helper(Some(ReplHelper::new(Self::possible_completions())));

        loop {
            let readline = rl.readline(&self.prompt());
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    if self.execute_command(line).await {
                        break;
                    }
                },
                Err(ReadlineError::Interrupted) => {},
                Err(ReadlineError::Eof) => break,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }

    fn prompt(&self) -> String {
        match &self.device {
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

    async fn connect<'a, T>(&mut self, words: &mut T) -> Result<ShouldExit> where
        T: Iterator<Item=&'a str> {
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

        let devices = self.manager.get_devices()?;
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

        self.device = Some(device);
        self.message_queue = Some(message_queue);

        Ok(false)
    }

    async fn devices<'a, T>(&self, words: &mut T) -> Result<ShouldExit> where
        T: Iterator<Item=&'a str> {
        if let Some(_) = words.next() {
            println!("devices: too many arguments, expected 0");
            return Ok(false);
        }

        let devices = self.manager.get_devices()?;

        for dev in devices {
            println!("{}", dev);
        }

        Ok(false)
    }

    async fn send<'a, T>(&self, words: &mut T) -> Result<ShouldExit> where
        T: Iterator<Item=&'a str> {
        let message_queue = match &self.message_queue {
            Some(s) => s,
            None => {
                println!("send: not connected to a device");
                return Ok(false);
            },
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

    async fn quit<'a, T>(&mut self, words: &mut T) -> Result<ShouldExit> where
        T: Iterator<Item=&'a str> {
        match words.next() {
            Some(_) => {
                println!("quit: too many arguments, expected 0");
                Ok(false)
            },
            None => Ok(true)
        }
    }
}
