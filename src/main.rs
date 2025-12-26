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
        return Err("bad label".to_string());
    }
    if msg.value != expected {
        return Err("bad value".to_string());
    }
    let pong = Message {
        label: MsgLabel::Pong,
        value: msg.value,
    };
    return Ok(pong);
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

        println!("A: sent {:?}...", &ping);
        if tx.send(ping).await.is_err() {
            println!("A: B dissapeared");
            break;
        }

        // wait for either pong or timeout
        let recieved = timeout(Duration::from_millis(200), ry.recv()).await;
        match recieved {
            Ok(Some(msg)) => {
                if msg.label != MsgLabel::Pong || msg.value != i {
                    println!("A: bad pong {:?}", msg);
                    break;
                }
                println!("A: got  {:?}", msg)
            }
            Ok(None) => {
                println!("A: channel closed?");
                break;
            }
            Err(_) => {
                println!("A: timeout mafaka");
                break;
            }
        }
    }
}
async fn task_b(mut rx: mpsc::Receiver<Message>, ty: mpsc::Sender<Message>) {
    //the pingee and ponger
    let mut expected: u32 = 0;
    while let Some(msg) = rx.recv().await {
        println!("B: got  {:?}!", msg);
        match on_ping(expected, msg) {
            Ok(pong) => {
                expected += 1;
                ty.send(pong).await.unwrap();
            }
            Err(e) => {
                println!("B: {e}");
                break;
            }
        }
    }
}
