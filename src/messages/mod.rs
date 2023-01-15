mod block_info;
mod file_info;
mod hello;

pub use block_info::{BlockInfo, BlockInfoDecoder, BlockInfoEncoder};
pub use file_info::{FileInfo, FileInfoDecoder, FileInfoEncoder};
pub use hello::{HelloMessage, HelloMessageDecoder, HelloMessageEncoder};

#[cfg(test)]
mod tests {
    use super::{
        block_info::{BlockInfoDecoder, BlockInfoEncoder},
        file_info::{FileInfo, FileInfoDecoder, FileInfoEncoder},
        BlockInfo, HelloMessage, HelloMessageDecoder, HelloMessageEncoder,
    };
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn hello_message() {
        // Create object.
        let hello_message = HelloMessage::new(123, "test".to_string(), "0.1.0".to_string());

        // Encode object.
        let mut hello_message_encoder = HelloMessageEncoder;
        let mut buffer = BytesMut::new();
        hello_message_encoder
            .encode(&hello_message, &mut buffer)
            .unwrap();

        // Decode object.
        let mut hello_message_decoder = HelloMessageDecoder;
        let decoded_hello_message = hello_message_decoder.decode(&mut buffer).unwrap().unwrap();

        // Make sure that we don't have unused bytes on the buffer.
        assert!(buffer.is_empty());

        // Make sure both objects are equal.
        assert_eq!(decoded_hello_message, hello_message);
    }

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
        let file_info = FileInfo::new(123, "/home/bob/foo/bar".into(), 123, 321);

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
}
