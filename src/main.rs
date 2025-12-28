mod protocol;
use crate::protocol::{Packet, ReceiverAction, SAWReceiver, SAWSender};
use std::time::Duration;
use tokio::{sync::mpsc, time::timeout};

fn main() {
    println!("--------------------start--------------------");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(run_async()).unwrap();

    println!("--------------------end--------------------");
}

async fn run_async() -> Result<(), String> {
    let (to_b, from_a) = mpsc::channel::<Packet>(8); // sends from a to b
    let (to_a, from_b) = mpsc::channel::<Packet>(8); // sends from b to a
    let (ra, rb) = tokio::join!(task_a(to_b, from_b), task_b(from_a, to_a));
    ra?;
    rb?;
    Ok(())
}

async fn task_a(
    to_b: mpsc::Sender<Packet>,
    mut from_b: mpsc::Receiver<Packet>,
) -> Result<(), String> {
    Ok(())
}

async fn task_b(
    mut from_a: mpsc::Receiver<Packet>,
    to_a: mpsc::Sender<Packet>,
) -> Result<(), String> {
    let mut rx = SAWReceiver::new();
    while let Some(pkt) = from_a.recv().await {
        match rx.on_packet(&pkt) {
            ReceiverAction::Send(ack) => to_a
                .send(ack)
                .await
                .map_err(|_e| "B:channel closed".to_string())?,
            ReceiverAction::Ignore => {}
            ReceiverAction::Error(e) => return Err(e),
        }
    }
    Ok(())
}
