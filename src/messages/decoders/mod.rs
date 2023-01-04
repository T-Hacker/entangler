mod hello;
mod message_type;
mod string;

pub use hello::HelloMessageDecoder;
pub use message_type::MessageTypeDecoder;
pub use string::StringDecoder;

const MAX_FRAME_SIZE: usize = 8 * 1024 * 1024;
