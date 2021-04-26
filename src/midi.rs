pub use midi::{Channel, Message};
use midi::{RawMessage, ToRawMessages};

pub trait MessageExt {
  fn encode(&self) -> Vec<u8>;
}

impl MessageExt for Message {
  fn encode(&self) -> Vec<u8> {
    let mut dest = Vec::new();
    for msg in self.to_raw_messages().into_iter() {
      match msg {
        RawMessage::Status(a) => dest.extend_from_slice(&[a | 0x80]),
        RawMessage::StatusData(a, b) => dest.extend_from_slice(&[a | 0x80, b]),
        RawMessage::StatusDataData(a, b, c) => dest.extend_from_slice(&[a | 0x80, b, c]),
        RawMessage::Raw(a) => dest.extend_from_slice(&[a]),
      }
    }
    dest
  }
}
