pub trait Deserializer {
    type InputEvent;
    type Error: std::error::Error;
    fn deserialize(&mut self, event: &[u8]) -> Result<Self::InputEvent, Self::Error>;
}
