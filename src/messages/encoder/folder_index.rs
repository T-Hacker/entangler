use crate::index::FolderIndex;
use bytes::BufMut;
use tokio_util::codec::Encoder;

use super::file_info::FileInfoEncoder;

pub struct FolderIndexEncoder;

impl Encoder<&FolderIndex> for FolderIndexEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &FolderIndex, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let entries = item.entries();

        // Write number of index entries.
        dst.put_u64_le(entries.len() as u64);

        // Write index entries.
        let mut file_info_encoder = FileInfoEncoder;
        for entry in entries.values() {
            file_info_encoder.encode(entry, dst)?;
        }

        Ok(())
    }
}
