type PartialResult<T, E> = Result<T, Result<(T, E), E>>;

#[derive(Clone, Debug)]
pub enum EventStatus {
    Success,
    Partial,
    Failure,
}

impl<T, E> From<&PartialResult<T, E>> for EventStatus {
    fn from(r: &PartialResult<T, E>) -> Self {
        match r {
            Ok(_) => EventStatus::Success,
            Err(Ok((_, _))) => EventStatus::Partial,
            Err(Err(_)) => EventStatus::Failure,
        }
    }
}

impl EventStatus {
    fn to_str(&self) -> &'static str {
        match self {
            EventStatus::Success => "success",
            EventStatus::Partial => "partial",
            EventStatus::Failure => "failure",
        }
    }
}
