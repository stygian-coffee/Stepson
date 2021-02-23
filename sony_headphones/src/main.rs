pub mod bluetooth;
pub mod message;
pub mod message_queue;
pub mod repl;
pub mod serializable;

use repl::Repl;

#[tokio::main]
async fn main() {
    let mut repl = Repl::new().unwrap();
    repl.run().await.unwrap();
}
