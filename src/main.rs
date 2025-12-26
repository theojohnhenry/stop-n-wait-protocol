use std::time::Duration;
use tokio::{sync::mpsc, time::timeout};

fn main() {
    println!("start");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(run_async());

    println!("end");
}

#[derive(Debug, PartialEq)]
enum MsgLabel {
    Ping,
    Pong,
}

#[derive(Debug)]
struct Message {
    label: MsgLabel,
    value: u32,
}

fn on_ping(expected: u32, msg: Message) -> Result<Message, String> {
    if msg.label != MsgLabel::Ping {
        return Err(format!("bad label: expected ping, got {:?}", msg.label));
    }
    if msg.value != expected {
        return Err(format!("bad value: expected {expected}, got {}", msg.value));
    }
    let pong = Message {
        label: MsgLabel::Pong,
        value: msg.value,
    };
    return Ok(pong);
}

fn on_pong(expected: u32, msg: Message) -> Result<(), String> {
    if msg.label != MsgLabel::Pong {
        return Err(format!("bad label: expected pong, got {:?}", msg.label));
    }
    if msg.value != expected {
        return Err(format!("bad value: expected {expected}, got {}", msg.value));
    }
    Ok(())
}

async fn run_async() {
    let (tx, rx) = mpsc::channel::<Message>(8); // sends from a to b
    let (ty, ry) = mpsc::channel::<Message>(8); // sends from b to a
    tokio::join!(task_a(tx, ry), task_b(rx, ty));
}

async fn task_a(tx: mpsc::Sender<Message>, mut ry: mpsc::Receiver<Message>) {
    //the pinger and pongee
    for i in 0..5u32 {
        let ping = Message {
            label: MsgLabel::Ping,
            value: i,
        };

        if tx.send(ping).await.is_err() {
            println!("A: B dissapeared");
            break;
        }

        // wait for either pong or timeout
        let received = timeout(Duration::from_millis(200), ry.recv()).await;
        match received {
            Err(e) => {
                println!("A: {e}");
                break;
            }
            Ok(Some(msg)) => {
                if let Err(e) = on_pong(i, msg) {
                    println!("A: {e}");
                    break;
                }
                println!("A: got pong {i}");
            }
            Ok(_) => {
                println!("A: channel is closed");
                break;
            }
        }
    }
}

async fn task_b(mut rx: mpsc::Receiver<Message>, ty: mpsc::Sender<Message>) {
    //the pingee and ponger
    let mut expected: u32 = 0;
    while let Some(msg) = rx.recv().await {
        match on_ping(expected, msg) {
            Ok(pong) => {
                if ty.send(pong).await.is_err() {
                    println!("B: Channel is closed");
                    break;
                }
                println!("B: got ping {expected}");
                expected += 1;
            }
            Err(e) => {
                println!("B: {e}");
                break;
            }
        }
    }
}
