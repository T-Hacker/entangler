mod block_info;
mod file_info;
mod watcher;

pub use block_info::{BlockInfo, BlockInfoDecoder, BlockInfoEncoder};
pub use file_info::{FileInfo, FileInfoDecoder, FileInfoEncoder};

use self::watcher::{WatcherEventDecoder, WatcherEventEncoder};
use bytes::{Buf, BufMut};
use std::io::ErrorKind;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum Message {
    WatcherEvent(notify::Event),
}

pub struct MessageEncoder;

impl Encoder<&Message> for MessageEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &Message, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        match item {
            Message::WatcherEvent(event) => {
                dst.put_u8(0);

                let mut watcher_event_encoder = WatcherEventEncoder;
                watcher_event_encoder.encode(event, dst)?;
            }
        }

        Ok(())
    }
}

pub struct MessageDecoder;

impl Decoder for MessageDecoder {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Read the message type.
        if src.is_empty() {
            src.reserve(1_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let message = src.get_u8();
        let message = match message {
            0 => {
                let mut watcher_event_decoder = WatcherEventDecoder;
                let Some(watcher_event) = watcher_event_decoder.decode(src)? else {
                    return Ok(None);
                };

                Message::WatcherEvent(watcher_event)
            }

            _ => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Invalid message type deceived: {message}",
                ))
            }
        };

        Ok(Some(message))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        block_info::{BlockInfoDecoder, BlockInfoEncoder},
        file_info::{FileInfo, FileInfoDecoder, FileInfoEncoder},
        watcher::{WatcherEventDecoder, WatcherEventEncoder},
        BlockInfo,
    };
    use bytes::BytesMut;
    use notify::{
        event::{CreateKind, EventAttributes},
        Event, EventKind,
    };
    use std::time::SystemTime;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn block_info() {
        // Create object.
        let block_info = BlockInfo::new(1, 123, 321, 121212);

        // Encode object.
        let mut block_info_encoder = BlockInfoEncoder;
        let mut buffer = BytesMut::new();
        block_info_encoder.encode(&block_info, &mut buffer).unwrap();

        // Decode object.
        let mut block_info_decoder = BlockInfoDecoder;
        let decoded_block_info = block_info_decoder.decode(&mut buffer).unwrap().unwrap();

        // Make sure that we don't have unused bytes on the buffer.
        assert!(buffer.is_empty());

        // Make sure both objects are equal.
        assert_eq!(decoded_block_info, block_info);
    }

    #[test]
    fn file_info() {
        // Create object.
        let file_info = FileInfo::new(123, "/home/bob/foo/bar".into(), 123, 321, SystemTime::now());

        // Encode object.
        let mut file_info_encoder = FileInfoEncoder;
        let mut buffer = BytesMut::new();
        file_info_encoder.encode(&file_info, &mut buffer).unwrap();

        // Decode object.
        let mut file_info_decoder = FileInfoDecoder;
        let decoded_file_info = file_info_decoder.decode(&mut buffer).unwrap().unwrap();

        // Make sure that we don't have unused bytes on the buffer.
        assert!(buffer.is_empty());

        // Make sure both objects are equal.
        assert_eq!(decoded_file_info, file_info);
    }

    #[test]
    fn watcher() {
        // Create object.
        let kind = EventKind::Create(CreateKind::File);
        let paths = vec!["/home/bob/foo/bar".into()];
        let event = Event {
            kind,
            paths,
            attrs: EventAttributes::new(),
        };

        // Encode object.
        let mut watcher_event_encoder = WatcherEventEncoder;
        let mut buffer = BytesMut::new();
        watcher_event_encoder.encode(&event, &mut buffer).unwrap();

        // Decode object.
        let mut watcher_event_decoder = WatcherEventDecoder;
        let decoded_event = watcher_event_decoder.decode(&mut buffer).unwrap().unwrap();

        // Make sure that we don't have unused bytes on the buffer.
        assert!(buffer.is_empty());

        // Make sure both objects are equal.
        assert_eq!(decoded_event, event);
    }
}
