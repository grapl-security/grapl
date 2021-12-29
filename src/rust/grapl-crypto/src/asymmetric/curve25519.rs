use rand_core::{OsRng, RngCore};
use x25519_dalek::{PublicKey, StaticSecret, SharedSecret};
use crate::aead::xchacha_blake3::{ChaChaBlake3, EncryptedData, AeadError, Aad};

#[derive(thiserror::Error, Debug)]
pub enum AsymmetricError {
    #[error("Aead")]
    AeadError(#[from] AeadError),
    #[error("InvalidPubkey")]
    InvalidPubkey,
    #[error("InvalidAadFormat")]
    InvalidAad(#[from] serde_json::Error)
}

pub struct PubEncryptedData {
    encrypted_data: EncryptedData,
}

pub struct Encrypter {
    their_public_key: PublicKey,
    aead_enc: ChaChaBlake3,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PubAad<A: Aad> {
    #[serde(bound = "A: Aad")]
    inner_aad: A,
    public_key: Vec<u8>
}

impl<A: Aad> Aad for PubAad<A> {
    fn recipient(&self) -> &str {
        self.inner_aad.recipient()
    }

    fn sender(&self) -> &str {
        self.inner_aad.sender()
    }
}

impl Encrypter {
    pub fn encrypt<A: Aad>(&mut self, msg: Vec<u8>, aad: A) -> Result<PubEncryptedData, AsymmetricError> {
        let (ephemeral_public_key, shared_secret) = self.new_ephemeral_keypair();
        let aad = PubAad {
            inner_aad: aad,
            public_key: ephemeral_public_key.as_bytes().to_vec()
        };
        let encrypted = self.aead_enc.encrypt(
            msg,
            aad,
            shared_secret.as_bytes(),
        )?;
        Ok(PubEncryptedData {
            encrypted_data: encrypted,
        })
    }

    pub(crate) fn new_ephemeral_keypair(
        &self,
    ) -> (PublicKey, SharedSecret) {
        let mut bytes: [u8; 32] = [0; 32];
        OsRng.fill_bytes(&mut bytes);
        // Note: We can't use EphemeralKey due to a version mismatch with rand_core
        // - StaticSecret is identical except it doesn't prevent re-use
        //   so just be extra sure to only use it once!
        let ephemeral_secret_key = StaticSecret::from(bytes);
        let ephemeral_public_key = PublicKey::from(&ephemeral_secret_key);
        let shared_secret = ephemeral_secret_key.diffie_hellman(&self.their_public_key);

        drop(ephemeral_secret_key);
        (ephemeral_public_key, shared_secret)
    }
}

pub struct Decrypter {
    our_secret: StaticSecret,
    aead_enc: ChaChaBlake3,
}

impl Decrypter {
    pub fn decrypt<A: Aad>(
        &mut self,
        encrypted_data: PubEncryptedData,
    ) -> Result<(Vec<u8>, PubAad<A>), AsymmetricError> {
        let aad: PubAad<A> = serde_json::from_slice(&encrypted_data.encrypted_data.aad)?;

        // Checking early here lets us avoid cloning if the length is invalid
        if aad.public_key.len() != 32 {
            return Err(AsymmetricError::InvalidPubkey)
        }

        let public_key: [u8; 32] = <[u8; 32]>::try_from(aad.public_key.clone()).map_err(|_| {
            AsymmetricError::InvalidPubkey
        })?;
        let shared_secret = self.our_secret.diffie_hellman(
            &PublicKey::from(public_key),
        );

        let decrypted = self.aead_enc.decrypt(
            encrypted_data.encrypted_data,
            shared_secret.as_bytes(),
        )?;

        Ok((decrypted, aad))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
    struct MyAad {
        recipient: String,
        sender: String,
        other_data: Vec<u8>,
    }

    impl Aad for MyAad {
        fn recipient(&self) -> &str {
            &self.recipient
        }

        fn sender(&self) -> &str {
            &self.sender
        }
    }

    fn rand_array<const N: usize>() -> [u8; N] {
        let mut n = [0; N];
        let mut rng = OsRng {};
        rng.fill_bytes(&mut n);
        n
    }

    #[quickcheck]
    fn encrypt_decrypt(msg: Vec<u8>, aad: Vec<u8>) {
        let aead = ChaChaBlake3::new();
        if msg.is_empty() { return; }

        let receiver_secret = StaticSecret::from(rand_array());
        let receiver_public = PublicKey::from(&receiver_secret);

        let aad = MyAad {
            recipient: "encrypt_decrypt_recipient".to_owned(),
            sender: "encrypt_decrypt_sender".to_owned(),
            other_data: aad,
        };

        let mut encrypter = Encrypter {
            their_public_key: receiver_public.clone(),
            aead_enc: aead.clone(),
        };

        let mut decrypter = Decrypter {
            our_secret: receiver_secret.clone(),
            aead_enc: aead.clone(),
        };

        let encrypted = encrypter.encrypt(msg.clone(), aad.clone()).expect("encrypt failed");
        let (plaintext, dec_aad): (_, PubAad<MyAad>) = decrypter.decrypt(encrypted).expect("encrypt failed");
        assert_eq!(msg, plaintext);
        assert_eq!(aad.recipient, dec_aad.recipient());
        assert_eq!(aad.sender, dec_aad.sender());
        assert_eq!(aad, dec_aad.inner_aad);
    }
}