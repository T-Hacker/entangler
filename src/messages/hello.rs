use super::MessageType;

#[derive(Debug, PartialEq)]
pub struct HelloMessage {
    r#type: MessageType,

    magic_number: u32,
    name: String,
    version: String,
}

impl HelloMessage {
    pub fn new(magic_number: u32, name: String, version: String) -> Self {
        Self {
            r#type: MessageType::Hello,
            magic_number,
            name,
            version,
        }
    }

    pub fn r#type(&self) -> &MessageType {
        &self.r#type
    }

    pub fn magic_number(&self) -> u32 {
        self.magic_number
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}
