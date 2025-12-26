use std::time::Duration;
use tokio::{sync::mpsc, time::timeout};

fn main() {
    println!("--------------------start--------------------");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(run_async());

    println!("--------------------end--------------------");
}

#[derive(Debug, PartialEq, Clone)]
enum MsgLabel {
    Ping,
    Pong,
}

#[derive(Debug, Clone)]
struct Message {
    label: MsgLabel,
    value: u32,
}
impl Message {
    fn ping(seq: u32) -> Message {
        Message {
            label: MsgLabel::Ping,
            value: seq,
        }
    }
    fn pong(seq: u32) -> Message {
        Message {
            label: MsgLabel::Pong,
            value: seq,
        }
    }
}
/// checks if ping is correct and returns a pong result or an error string
fn on_ping(next_seq: u32, msg: &Message) -> Result<Message, String> {
    if msg.label != MsgLabel::Ping {
        return Err(format!("bad label: expected ping, got {:?}", msg.label));
    }
    if msg.value != next_seq {
        return Err(format!("bad value: expected {next_seq}, got {}", msg.value));
    }
    let pong = Message::pong(next_seq);
    return Ok(pong);
}

/// checks if pong is correct and returns an ok or an error string
fn on_pong(next_seq: u32, msg: &Message) -> Result<(), String> {
    if msg.label != MsgLabel::Pong {
        return Err(format!("bad label: expected pong, got {:?}", msg.label));
    }
    if msg.value != next_seq {
        return Err(format!("bad value: expected {next_seq}, got {}", msg.value));
    }
    Ok(())
}

async fn run_async() {
    let (to_b, from_a) = mpsc::channel::<Message>(8); // sends from a to b
    let (to_a, from_b) = mpsc::channel::<Message>(8); // sends from b to a
    tokio::join!(task_a(to_b, from_b), task_b(from_a, to_a));
}

async fn task_a(to_b: mpsc::Sender<Message>, mut from_b: mpsc::Receiver<Message>) {
    //the pinger and pongee
    for seq in 0..5u32 {
        let ping = Message::ping(seq);

        for attempt in 0..3u32 {
            println!("A: sending \n {:?}", &ping);
            if to_b.send(ping.clone()).await.is_err() {
                println!("A: B dissapeared");
                break;
            }
            // wait for either pong or timeout
            let received = timeout(Duration::from_millis(500), from_b.recv()).await;
            match received {
                Err(e) => {
                    println!("A: {e}"); // timed out error
                    continue;
                }
                Ok(Some(msg)) => {
                    println!("A: got \n {:?}", &msg);
                    if let Err(e) = on_pong(seq, &msg) {
                        println!("A: {e}");
                        break;
                    }
                    break;
                }
                Ok(_) => {
                    println!("A: channel is closed");
                    break;
                }
            }
        }
    }
}

async fn task_b(mut from_a: mpsc::Receiver<Message>, to_a: mpsc::Sender<Message>) {
    //the pingee and ponger
    let mut next_seq: u32 = 0;
    while let Some(msg) = from_a.recv().await {
        //handle recieved ping by responding with a pong
        match on_ping(next_seq, &msg) {
            Ok(pong) => {
                println!("B: got \n {:?}", &msg);
                next_seq += 1;
                //simulate packet loss. drop pong every third ping
                if pong.value % 2 == 0 && pong.value != 0 {
                    println!("SIM: B is dropping {:?}", &pong);
                    continue;
                }
                println!("B: sending \n {:?}", pong);
                if to_a.send(pong).await.is_err() {
                    println!("B: Channel is closed");
                    break;
                }
            }
            Err(e) => {
                println!("B: {e}");
                break;
            }
        }
    }
}
