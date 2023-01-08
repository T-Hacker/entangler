use super::FileInfoDecoder;
use crate::index::FolderIndex;
use bytes::{Buf, BytesMut};
use std::collections::HashMap;
use tokio_util::codec::Decoder;

pub struct FolderIndexDecoder;

impl Decoder for FolderIndexDecoder {
    type Item = FolderIndex;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check if we have data for deserializing the file info.
        if src.len() < 8 {
            src.reserve(8_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        // Read entires length.
        let entries_len = src.get_u64_le();

        // Deserialize file info.
        let mut file_info_decoder = FileInfoDecoder;
        let mut entries = HashMap::with_capacity(entries_len as usize);
        for _ in 0..entries_len {
            let Some(file_info) = file_info_decoder.decode(src)? else {
                return Ok(None);
            };

            entries.insert(file_info.path().to_owned(), file_info);
        }

        // Return deserialized object.
        Ok(Some(FolderIndex::with_entires(entries)))
    }
}
