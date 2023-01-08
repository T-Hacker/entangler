mod block_info;
mod file_info;
mod folder_index;
mod hello;
mod message_type;
mod string;

pub use block_info::BlockInfoDecoder;
pub use file_info::FileInfoDecoder;
pub use folder_index::FolderIndexDecoder;
pub use hello::HelloMessageDecoder;
pub use message_type::MessageTypeDecoder;
pub use string::StringDecoder;

const MAX_FRAME_SIZE: usize = 8 * 1024 * 1024;
