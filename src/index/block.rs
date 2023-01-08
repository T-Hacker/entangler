use sha3::{Digest, Sha3_256};

#[derive(Debug)]
pub struct Block {
    offset: u64,
    block_size: u32,
    crc32: u32,
    sha3: [u8; 32],
}

impl Block {
    pub fn new(offset: u64, buffer: &[u8]) -> Self {
        let block_size = buffer.len() as u32;

        // Calculate CRC32.
        let crc32 = crc32fast::hash(buffer);

        // Calculate SHA3.
        let mut hasher = Sha3_256::new();
        hasher.update(buffer);
        let sha3 = hasher.finalize();
        let sha3 = sha3.as_slice().try_into().unwrap();

        // Create block object.
        Self {
            offset,
            block_size,
            crc32,
            sha3,
        }
    }
}