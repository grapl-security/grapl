use blake3::Hash;
use x25519_dalek::{PublicKey, StaticSecret, SharedSecret};
use crate::aead::xchacha_blake3::{ChaChaBlake3, EncryptedData, AeadError};
use crate::hasher::Hasher;

const SYMKEY_ENCRYPT: &'static [u8] = b"Grapl-12211946";

#[derive(thiserror::Error, Debug)]
pub enum AsymmetricError {
    #[error("Aead")]
    AeadError(#[from] AeadError),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PubEncryptedData {
    encrypted_data: EncryptedData,
    #[serde(with = "crate::arrays")]
    public_key: [u8; 32],
    // The salt used to derive the symmetric key
    #[serde(with = "crate::arrays")]
    salt: [u8; 16],
}

pub struct Encrypter {
    their_public_key: PublicKey,
    aead_enc: ChaChaBlake3,
    hasher: Hasher,
}

impl Encrypter {
    pub fn encrypt(&mut self, msg: Vec<u8>, aad: &[u8]) -> Result<PubEncryptedData, AsymmetricError> {
        let (ephemeral_public_key, shared_secret) = self.new_ephemeral_keypair();
        let salt = crate::rand_bytes::rand_array::<32, 16>();
        let derived_key = derive_key(
            &mut self.hasher,
            shared_secret,
            &ephemeral_public_key,
            &salt,
        );
        let encrypted = self.aead_enc.encrypt(
            msg,
            aad,
            derived_key.as_bytes(),
        )?;

        Ok(PubEncryptedData {
            encrypted_data: encrypted,
            public_key: ephemeral_public_key.to_bytes(),
            salt,
        })
    }

    pub(crate) fn new_ephemeral_keypair(
        &self,
    ) -> (PublicKey, SharedSecret) {
        let bytes = crate::rand_bytes::rand_array::<64, 32>();
        // Note: We can't use EphemeralKey due to a version mismatch with rand_core
        // - StaticSecret is identical except it doesn't prevent re-use
        //   so just be extra sure to only use it once!
        let ephemeral_secret_key = StaticSecret::from(bytes);
        let ephemeral_public_key = PublicKey::from(&ephemeral_secret_key);
        let shared_secret = ephemeral_secret_key.diffie_hellman(&self.their_public_key);
        // Explicitly drop the ephemeral_secret_key - there should never be a reason to remove this line!
        drop(ephemeral_secret_key);
        (ephemeral_public_key, shared_secret)
    }
}

pub struct Decrypter {
    our_secret: StaticSecret,
    aead_enc: ChaChaBlake3,
    hasher: Hasher,
}

impl Decrypter {
    pub fn decrypt(
        &mut self,
        encrypted_data: PubEncryptedData,
    ) -> Result<Vec<u8>, AsymmetricError> {
        let public_key = PublicKey::from(encrypted_data.public_key);
        let shared_secret = self.our_secret.diffie_hellman(
            &public_key,
        );

        let derived_key = derive_key(
            &mut self.hasher,
            shared_secret,
            &public_key,
            &encrypted_data.salt,
        );

        let decrypted = self.aead_enc.decrypt(
            encrypted_data.encrypted_data,
            derived_key.as_bytes(),
        )?;

        Ok(decrypted)
    }
}

// Q: Why do we hash the shared secret before using it for encryption?
// A: The shared secret is not a cryptographic hash, it is a Montgomery point.
//    As such, its bits are not randomly distributed. Hashing the value randomly distributes
//    the bits. This means that any quirky properties of the the point's bit distribution
//    are washed away.
// Note: The curve25519 group is not of prime order.
fn derive_key(
    hasher: &mut Hasher,
    shared_secret: SharedSecret,
    public_key: &PublicKey,
    salt: &[u8; 16],
) -> Hash {
    let derived_key = hasher.hash(|hasher| {
        hasher.update(shared_secret.as_bytes());
        hasher.update(SYMKEY_ENCRYPT);
        hasher.update(salt.as_ref());
        hasher.update(&public_key.as_bytes()[..]);
    });

    derived_key
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
            hasher: Default::default()
        };

        let mut decrypter = Decrypter {
            our_secret: receiver_secret.clone(),
            aead_enc: aead.clone(),
            hasher: Default::default()
        };

        let encrypted = encrypter.encrypt(msg.clone(), &aad).expect("encrypt failed");
        let decrypted = decrypter.decrypt(encrypted).expect("encrypt failed");
        assert_eq!(msg, decrypted);
    }
}