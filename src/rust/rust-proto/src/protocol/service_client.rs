/// Equivalent to the NamedService trait in tonic for server constructs.
/// Pass this NAME to e.g. a healthcheck client.
pub trait NamedService {
    const SERVICE_NAME: &'static str;
}
