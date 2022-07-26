use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct LensSubscriptionUrl(String);

impl From<String> for LensSubscriptionUrl {
    fn from(u: String) -> Self {
        Self(u)
    }
}

impl FromStr for LensSubscriptionUrl {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

impl LensSubscriptionUrl {
    pub fn inner(self) -> String {
        self.0
    }

    pub fn get_ref(&self) -> &str {
        &self.0
    }
}
