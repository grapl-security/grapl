use rand_core::{OsRng, RngCore};
use x25519_dalek::{PublicKey, StaticSecret, SharedSecret};
use crate::aead::xchacha_blake3::{ChaChaBlake3, EncryptedData, AeadError};

#[derive(thiserror::Error, Debug)]
pub enum AsymmetricError {
    #[error("Aead")]
    AeadError(#[from] AeadError),
}

pub struct PubEncryptedData {
    encrypted_data: EncryptedData,
    public_key: PublicKey,
}

pub struct Encrypter {
    their_public_key: PublicKey,
    aead_enc: ChaChaBlake3,
}

impl Encrypter {
    pub fn encrypt(&mut self, msg: Vec<u8>, aad: &[u8]) -> Result<PubEncryptedData, AsymmetricError> {
        let (ephemeral_public_key, shared_secret) = self.new_ephemeral_keypair();
        let encrypted = self.aead_enc.encrypt(
            msg,
            aad,
            shared_secret.as_bytes(),
        )?;
        Ok(PubEncryptedData {
            encrypted_data: encrypted,
            public_key: ephemeral_public_key,
        })
    }

    pub(crate) fn new_ephemeral_keypair(
        &self,
    ) -> (PublicKey, SharedSecret) {
        let mut bytes: [u8; 32] = [0; 32];
        OsRng.fill_bytes(&mut bytes);
        // Note: We can't use EphemeralKey due to a version mismatch with rand_core
        // - StaticSecret is identitical except it doesn't prevent re-use
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
    pub fn decrypt(
        &mut self,
        encrypted_data: PubEncryptedData,
    ) -> Result<Vec<u8>, AsymmetricError> {
        let shared_secret = self.our_secret.diffie_hellman(
            &encrypted_data.public_key,
        );

        let decrypted = self.aead_enc.decrypt(
            encrypted_data.encrypted_data,
            shared_secret.as_bytes(),
        )?;
        Ok(decrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;


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

        let mut encrypter = Encrypter {
            their_public_key: receiver_public.clone(),
            aead_enc: aead.clone(),
        };

        let mut decrypter = Decrypter {
            our_secret: receiver_secret.clone(),
            aead_enc: aead.clone(),
        };

        let encrypted = encrypter.encrypt(msg.clone(), aad).expect("encrypt failed");
        let decrypted = decrypter.decrypt(encrypted).expect("encrypt failed");
        assert_eq!(msg, decrypted);
    }
}