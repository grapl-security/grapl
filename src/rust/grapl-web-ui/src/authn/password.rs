use std::fmt::{Display, Formatter};
use argon2::{
    Argon2,
    Algorithm,
    Params,
    PasswordHash,
    PasswordVerifier,
    Version,
};

pub struct Password(String);

impl Password {
    #[tracing::instrument(skip(self))]
    pub fn verify_hash(&self, hash_to_verify: &str) -> Result<(), argon2::password_hash::Error> {
        // IMPORTANT: Keep in sync w/ https://github.com/grapl-security/grapl/blob/main/src/python/provisioner/provisioner/app.py#L84
        let password_hasher = Argon2::new(Algorithm::Argon2i, Version::V0x13, Params::new(
            102400, 2, 8, None
        )?);

        let hash = PasswordHash::new(hash_to_verify)?;

        password_hasher.verify_password(self.0.as_bytes(), &hash)
    }
}

impl From<String> for Password {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("<password value hidden>").finish()
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("<password value hidden>").finish()
    }
}