use std::io::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Data { seq: u32 },
    Ack { seq: u32 },
}

///Stop-and-wait sender
pub struct SAWSender {
    seq: u32,
    max_seq: u32,
    attempt: u32,
    max_attempts: u32,
}
impl SAWSender {
    pub fn new(max_seq: u32, max_attempts: u32) -> Self {
        Self {
            seq: 0,
            max_seq,
            attempt: 0,
            max_attempts,
        }
    }
    pub fn done(&self) -> bool {
        self.seq >= self.max_seq
    }
    pub fn start_seq(&mut self) -> Packet {
        self.attempt = 1;
        Packet::Data { seq: self.seq }
    }
    pub fn on_timeout(&mut self) -> Result<Packet, String> {
        // do i have attempts left?
        // yes -retry increase attempts
        // no -return error
        if self.attempt >= self.max_attempts {
            return Err(format!("gave up on {:?}", &self.seq));
        }
        self.attempt += 1;
        Ok(Packet::Data { seq: (self.seq) })
    }
    pub fn on_ack(&mut self, packet: &Packet) -> Result<(), String> {
        // check if it matches my seq and that its an ack
        // if it doesnt return error.
        // if it does increase seq and reset attempt
        match packet {
            Packet::Ack { seq } => {
                if *seq != self.seq {
                    return Err(format!(
                        "Incorrect sequence. Expected: {}, got: {}",
                        self.seq, seq
                    ));
                }
                self.seq += 1;
                self.attempt = 0;
                Ok(())
            }
            other => Err(format!("Incorrect ACK. Got {:?}", other)),
        }
    }
}
#[derive(Debug)]
pub enum ReceiverAction {
    Send(Packet),
    Ignore,
    Error(String),
}
pub struct SAWReceiver {
    next_seq: u32,
    last_ack: Option<Packet>,
}
impl SAWReceiver {
    pub fn new() -> Self {
        SAWReceiver {
            next_seq: 0,
            last_ack: None,
        }
    }
    pub fn on_packet(&mut self, packet: &Packet) -> ReceiverAction {
        // accept new data - seq==next_seq - advance and send ack
        // duplicate data - seq+1 == next_seq - resend last ack
        // everything else - error

        match packet {
            Packet::Data { seq } => {
                if *seq == self.next_seq {
                    let new_ack = Packet::Ack { seq: *seq };
                    self.last_ack = Some(new_ack.clone());
                    self.next_seq += 1;
                    return ReceiverAction::Send(new_ack);
                } else if *seq + 1 == self.next_seq {
                    match &self.last_ack {
                        Some(ack) => {
                            return ReceiverAction::Send(ack.clone());
                        }
                        None => return ReceiverAction::Error("snopp".to_string()),
                    }
                } else {
                    return ReceiverAction::Error("snopp".to_string());
                }
            }
            other => ReceiverAction::Error("knulla".to_string()),
        }
    }
}
