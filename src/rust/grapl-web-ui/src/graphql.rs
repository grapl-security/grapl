use std::str::FromStr;

// We have a new type for this to differentiate between the URL for this backend service and that
// for others
#[derive(Clone, Debug)]
pub struct GraphQlEndpointUrl(url::Url);

impl From<url::Url> for GraphQlEndpointUrl {
    fn from(u: url::Url) -> Self {
        Self(u)
    }
}

impl FromStr for GraphQlEndpointUrl {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<url::Url>().map(GraphQlEndpointUrl::from)
    }
}

impl GraphQlEndpointUrl {
    pub fn inner(self) -> url::Url {
        self.0
    }

    pub fn get_ref(&self) -> &url::Url {
        &self.0
    }
}
