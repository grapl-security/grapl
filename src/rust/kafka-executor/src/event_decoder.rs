pub trait EventDeserializer {
    type InputEvent;
    type Error: std::error::Error;
    fn decode_event(&self, event: &[u8]) -> Result<Self::InputEvent, Self::Error>;
}
