// We have a new type for this to differentiate between the URL for this backend service and that
// for others
#[derive(Clone, Debug)]
pub struct GraphQlEndpointUrl(url::Url);

impl From<url::Url> for GraphQlEndpointUrl {
    fn from(u: url::Url) -> Self {
        Self(u)
    }
}

impl std::ops::Deref for GraphQlEndpointUrl {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
