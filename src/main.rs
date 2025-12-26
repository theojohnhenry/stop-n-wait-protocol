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
    let mut i: u32 = 0;
    while let Some(msg) = rx.recv().await {
        println!("B: got  {:?}!", msg);
        println!("B: checking message");
        if msg.value != i || msg.label != MsgLabel::Ping {
            panic!("B: bad ping")
        }

        i += 1;
        println!("right iteration!");

        let pong = Message {
            label: MsgLabel::Pong,
            value: msg.value,
        };
        println!("B: sent {:?}...", &pong);
        ty.send(pong).await.unwrap();
    }
}
