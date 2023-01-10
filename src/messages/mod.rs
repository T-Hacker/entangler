pub mod decoders;
pub mod encoder;

mod hello;

// Exported types.
pub use hello::HelloMessage;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum MessageType {
    Hello = 0,
    Index,
}

#[cfg(test)]
mod tests {
    use super::{
        decoders::{BlockInfoDecoder, FileInfoDecoder, HelloMessageDecoder, StringDecoder},
        encoder::{BlockInfoEncoder, FileInfoEncoder, HelloMessageEncoder, StringEncoder},
        hello::HelloMessage,
    };
    use crate::{
        index::{BlockInfo, FileInfo, FolderIndexBuilder},
        messages::{decoders::FolderIndexDecoder, encoder::FolderIndexEncoder},
        MAGIC_NUMBER, NAME, VERSION,
    };
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn hello_message() {
        // Create message object.
        let magic_number = MAGIC_NUMBER;
        let name = NAME.to_string();
        let version = VERSION.to_string();
        let message = HelloMessage::new(magic_number, name, version);

        // Encode message object.
        let mut encoder = HelloMessageEncoder;
        let mut buffer = BytesMut::new();
        encoder.encode(&message, &mut buffer).unwrap();

        // Decode message object.
        let mut decoder = HelloMessageDecoder;
        let decoded_message = decoder.decode(&mut buffer).unwrap().unwrap();

        // Assert that we don't have more bytes in the buffer.
        assert!(buffer.is_empty());

        // Assert that both message are equal.
        assert_eq!(decoded_message, message);
    }

    #[test]
    fn string() {
        // Create string.
        let string = "This is a string!".to_string();

        // Encode string object.
        let mut encoder = StringEncoder;
        let mut buffer = BytesMut::new();
        encoder.encode(&string, &mut buffer).unwrap();

        // Decode string object.
        let mut decoder = StringDecoder;
        let decoded_string = decoder.decode(&mut buffer).unwrap().unwrap();

        // Assert that we don't have more bytes in the buffer.
        assert!(buffer.is_empty());

        // Assert that both strings are equal.
        assert_eq!(decoded_string, string);
    }

    #[test]
    fn block_info() {
        // Create block info object.
        let block_info = BlockInfo::new(123, 333, 323, [3; 32]);

        // Encode block info object.
        let mut encoder = BlockInfoEncoder;
        let mut buffer = BytesMut::new();
        encoder.encode(&block_info, &mut buffer).unwrap();

        // Decode block info object.
        let mut decoder = BlockInfoDecoder;
        let decoded_block_info = decoder.decode(&mut buffer).unwrap().unwrap();

        // Assert that we don't have more bytes in the buffer.
        assert!(buffer.is_empty());

        // Assert that both objects are equal.
        assert_eq!(decoded_block_info, block_info);
    }

    #[test]
    fn file_info() {
        // Create file info object.
        let blocks = vec![
            BlockInfo::new(123, 333, 323, [3; 32]),
            BlockInfo::new(123, 333, 323, [3; 32]),
            BlockInfo::new(123, 333, 323, [3; 32]),
        ];
        let file_info = FileInfo::new("/foo/bar".into(), 123, 333, blocks);

        // Encode file info object.
        let mut encoder = FileInfoEncoder;
        let mut buffer = BytesMut::new();
        encoder.encode(&file_info, &mut buffer).unwrap();

        // Decode file info object.
        let mut decoder = FileInfoDecoder;
        let decoded_file_info = decoder.decode(&mut buffer).unwrap().unwrap();

        // Assert that we don't have more bytes in the buffer.
        assert!(buffer.is_empty());

        // Assert that both objects are equal.
        assert_eq!(decoded_file_info, file_info);
    }

    #[tokio::test]
    async fn folder_index_builder() {
        tracing_subscriber::fmt().init();

        // Create folder index.
        let folder_index = FolderIndexBuilder::from_path(".").build().await;
        assert!(!folder_index.entries().is_empty());

        // Encode folder index.
        let mut encoder = FolderIndexEncoder;
        let mut buffer = BytesMut::new();
        encoder.encode(&folder_index, &mut buffer).unwrap();

        // Decode folder index.
        let mut decoder = FolderIndexDecoder;
        let decoded_folder_index = decoder.decode(&mut buffer).unwrap().unwrap();

        // Assert that we don't have more bytes in the buffer.
        assert!(buffer.is_empty());

        // Assert that both folder indices are equal.
        assert_eq!(folder_index, decoded_folder_index);
    }
}
