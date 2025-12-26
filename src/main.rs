use std::time::Duration;
use tokio::{sync::mpsc, time::timeout};

fn main() {
    println!("--------------------start--------------------");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(run_async()).unwrap();

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

async fn run_async() -> Result<(), String> {
    let (to_b, from_a) = mpsc::channel::<Message>(8); // sends from a to b
    let (to_a, from_b) = mpsc::channel::<Message>(8); // sends from b to a
    let (ra, rb) = tokio::join!(task_a(to_b, from_b), task_b(from_a, to_a));
    ra?;
    rb?;
    Ok(())
}

async fn task_a(
    to_b: mpsc::Sender<Message>,
    mut from_b: mpsc::Receiver<Message>,
) -> Result<(), String> {
    //the pinger and pongee
    for seq in 0..5u32 {
        let ping: Message = Message::ping(seq);
        let mut success: bool = false;

        for attempt in 1..4 {
            println!("A: sending {:?}. attempt nr {attempt}", &ping);
            to_b.send(ping.clone())
                .await
                .map_err(|_| "A: channel closed".to_string())?;

            let Ok(opt) = timeout(Duration::from_millis(200), from_b.recv()).await else {
                println!("A: timeout");
                continue;
            };
            let Some(msg) = opt else {
                return Err("A: channel closed".into());
            };
            println!("A: got  {:?}", &msg);
            on_pong(seq, &msg)?;
            success = true;
            break;
        }
        if !success {
            return Err(format!("A: gave up on seq {seq}"));
        }
    }
    Ok(())
}

async fn task_b(
    mut from_a: mpsc::Receiver<Message>,
    to_a: mpsc::Sender<Message>,
) -> Result<(), String> {
    //the pingee and ponger
    let mut next_seq: u32 = 0;
    let mut last_pong: Option<Message> = None;

    while let Some(msg) = from_a.recv().await {
        println!("B: got {:?}", &msg);
        if msg.label != MsgLabel::Ping {
            return Err(format!("B: Expected ping, got {:?}", msg.label));
        }
        let seq = msg.value;

        //case 1: new ping
        if seq == next_seq {
            let pong = Message::pong(seq);
            last_pong = Some(pong.clone());
            next_seq += 1;

            //simulate dropped pong
            if pong.value % 2 == 0 && pong.value != 0 {
                println!("SIM: B dropping pong {:?}", pong);
                continue;
            }

            println!("B: sending {:?}", &pong);
            to_a.send(pong)
                .await
                .map_err(|_| "B: channel closed".to_string())?;
            continue;
        }
        //case 2: duplicate ping aka retry
        if seq + 1 == next_seq {
            let pong = last_pong
                .clone()
                .ok_or_else(|| "B: duplicate ping but no last_pong yet".to_string())?;
            println!("B: duplicate ping {seq}, resending {:?}", pong);
            to_a.send(pong)
                .await
                .map_err(|_| "B: channel closed".to_string())?;
            continue;
        }
        //case 3: out of order?
        return Err(format!(
            "B: out of order ping: got {seq}, expected {next_seq}"
        ));
    }
    Ok(())
}
