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
    let (ra, rb) = tokio::join!(client_a(to_b, from_b), server_b(from_a, to_a));
    ra?;
    rb?;
    Ok(())
}

async fn client_a(
    to_b: mpsc::Sender<Packet>,
    mut from_b: mpsc::Receiver<Packet>,
) -> Result<(), String> {
    let mut tx = SAWSender::new(5, 3);
    let timeout_duration = Duration::from_millis(200);

    while !tx.done() {
        //try send packet
        let mut out_pkt = tx.start_seq();
        loop {
            println!("A: sending {:?}", out_pkt);
            to_b.send(out_pkt.clone())
                .await
                .map_err(|_| "A: send error".to_string())?;

            // wait for recieved packet
            let Ok(opt) = timeout(timeout_duration, from_b.recv()).await else {
                // timed out
                // do something - retry 3 times
                // on_timeout();
                out_pkt = tx.on_timeout()?;
                continue;
            };
            // didnt time out
            let Some(inc_pkt) = opt else {
                //didnt return a pkt ?
                // do something - return error;
                return Err("A: channel closed".to_string());
            };
            // all good, recieved ack
            // on_ack(); validate ack, then break loop, and advance internal seq

            println!("A: recieved {:?}", inc_pkt);
            tx.on_ack(&inc_pkt)?;
            break;
        }
    }
    Ok(())
}

async fn server_b(
    mut from_a: mpsc::Receiver<Packet>,
    to_a: mpsc::Sender<Packet>,
) -> Result<(), String> {
    let mut rx = SAWReceiver::new();
    while let Some(inc_pkt) = from_a.recv().await {
        println!("B: recieved {:?}", inc_pkt);
        match rx.on_packet(&inc_pkt) {
            ReceiverAction::Send(ack) => {
                println!("B sending {:?}", ack);
                to_a.send(ack)
                    .await
                    .map_err(|_e| "B:channel closed".to_string())?
            }
            ReceiverAction::Ignore => {}
            ReceiverAction::Error(e) => return Err(e),
        }
    }
    Ok(())
}
