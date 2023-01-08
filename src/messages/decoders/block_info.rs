use crate::index::BlockInfo;
use tokio_util::codec::Decoder;

pub struct BlockInfoDecoder;

impl Decoder for BlockInfoDecoder {
    type Item = BlockInfo;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}
