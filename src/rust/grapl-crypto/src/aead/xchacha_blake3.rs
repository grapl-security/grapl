#![allow(warnings)]
// #![feature(test)]

use std::ops::{Deref, DerefMut};
// use test::black_box;
use blake3::Hash;
use chacha20::{Key, XNonce, XChaCha12};
use chacha20::cipher::{NewCipher, StreamCipher};
use chacha20::cipher::errors::LoopError;
use rand_core::{OsRng, RngCore};

// These domains are used to derive two separate keys from the same origin key.
// The values can be arbitrary, but must be distinct.
const DOMAIN_ENCRYPT: &'static [u8] = b"Grapl-12211946";
const DOMAIN_AUTH: &'static [u8] = b"Grapl-02142019";

const KEY_SIZE: usize = 32;
const ENC_NONCE_SIZE: usize = 24;
const MIN_KEY_NONCE_SIZE: usize = 24;
const TOTAL_NONCE_SIZE: usize = ENC_NONCE_SIZE + MIN_KEY_NONCE_SIZE;

// This aead construct is XChaCha12 with BLAKE3
// * XChaCha12
// * BLAKE3

// Inspiration:
// https://mccarty.io/cryptography/2021/11/29/chacha20-blake3.html
// https://github.com/soatok/experimental-caead
// https://soatok.blog/2020/09/09/designing-new-cryptography-for-non-standard-threat-models/
// https://eprint.iacr.org/2019/1492.pdf

#[derive(Debug, thiserror::Error)]
pub enum AeadError {
    #[error("AuthenticationError")]
    AuthenticationError,
    #[error("LoopError")]
    LoopError
}

impl From<chacha20::cipher::errors::LoopError> for AeadError {
    fn from(_: LoopError) -> Self {
        Self::LoopError
    }
}

#[derive(Clone)]
struct Hasher {
    hasher: blake3::Hasher,
}

impl Default for Hasher {
    fn default() -> Self {
        Hasher { hasher: blake3::Hasher::new() }
    }
}

impl Hasher {
    fn hash<F>(&mut self, f: F) -> Hash
        where F: FnOnce(&mut blake3::Hasher)
    {
        self.hasher.reset();
        f(&mut self.hasher);
        self.hasher.finalize()
    }
}

#[derive(Clone)]
pub struct EncryptedData {
    ciphertext: Vec<u8>,
    pub(crate) aad: Vec<u8>,
    nonce: [u8; TOTAL_NONCE_SIZE],
    mac: [u8; KEY_SIZE],
}

/// ChaChaBlake3 is an Aead
#[derive(Clone)]
pub struct ChaChaBlake3 {
    cbuf: Vec<u8>,
    hasher: Hasher,
}

impl ChaChaBlake3 {
    pub fn new() -> Self {
        Self {
            cbuf: Vec::with_capacity(64),
            hasher: Hasher::default(),
        }
    }

    pub fn encrypt(
        &mut self,
        msg: Vec<u8>,
        aad: &[u8],
        key: &[u8; KEY_SIZE],
    ) -> Result<EncryptedData, AeadError> {
        let mut nonce = [0; TOTAL_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);

        let (ciphertext, mac) = self.encrypt_raw(
            msg,
            &nonce,
            aad,
            &key,
        )?;

        Ok(EncryptedData {
            ciphertext,
            nonce,
            aad: aad.to_vec(),
            mac: mac.into(),
        })
    }

    pub fn decrypt(
        &mut self,
        encrypted_data: EncryptedData,
        key: &[u8; KEY_SIZE],
    ) -> Result<Vec<u8>, AeadError> {
        debug_assert!(key.len() > 16);
        self.decrypt_raw(
            encrypted_data.ciphertext,
            &encrypted_data.mac.into(),
            &encrypted_data.nonce,
            &encrypted_data.aad,
            &key,
        )
    }

    pub(crate) fn encrypt_raw(
        &mut self,
        mut msg: Vec<u8>,
        nonce: &[u8; TOTAL_NONCE_SIZE],
        aad: &[u8],
        key: &[u8; KEY_SIZE],
    ) -> Result<(Vec<u8>, Hash), AeadError> {
        let (enc_key, auth_key) = self.split_keys(&nonce[ENC_NONCE_SIZE..], key);

        let enc_key = Key::from_slice(&enc_key);
        let enc_nonce = XNonce::from_slice(&nonce[..ENC_NONCE_SIZE]);
        let mut cipher = XChaCha12::new(&enc_key, &enc_nonce);
        cipher.try_apply_keystream(&mut msg)?;

        let mac: Hash = self.hmac(&aad, &msg[..], &auth_key);

        Ok((msg, mac))
    }

    pub(crate) fn decrypt_raw(
        &mut self,
        mut ciphertext: Vec<u8>,
        tag: &Hash,
        nonce: &[u8; TOTAL_NONCE_SIZE],
        aad: &[u8],
        key: &[u8; KEY_SIZE],
    ) -> Result<Vec<u8>, AeadError> {
        let (enc_key, auth_key) = self.split_keys(&nonce[ENC_NONCE_SIZE..], key);

        let mac: Hash = self.hmac(&aad, &ciphertext[..], &auth_key);

        // `mac` and `tag` are both `Hash`, which implements a cosntant time equality comparison
        if &mac != tag {
            return Err(AeadError::AuthenticationError)
        }

        let enc_key = Key::from_slice(&enc_key);
        let enc_nonce = XNonce::from_slice(&nonce[..ENC_NONCE_SIZE]);
        let mut cipher = XChaCha12::new(&enc_key, &enc_nonce);
        cipher.apply_keystream(&mut ciphertext);
        Ok(ciphertext)
    }

    fn hmac(&mut self, aad: &[u8], ciphertext: &[u8], auth_key: &[u8; KEY_SIZE]) -> Hash {
        let aad_length = aad.len().to_le_bytes();
        let ciphertext_length = ciphertext.len().to_le_bytes();

        self.cbuf.clear();

        self.cbuf.extend_from_slice(&aad[..]);
        self.cbuf.extend_from_slice(&ciphertext[..]);
        self.cbuf.extend_from_slice(&aad_length[..]);
        self.cbuf.extend_from_slice(&ciphertext_length[..]);

        self.hasher.hash(|hasher| {
            hasher.update(&auth_key[..]);
            hasher.update(self.cbuf.as_slice());
        })
    }

    // `split_keys` takes a root key and a nonce and generates two domain keys
    fn split_keys(&mut self, nonce: &[u8], key: &[u8; KEY_SIZE]) -> ([u8; KEY_SIZE], [u8; KEY_SIZE]) {
        assert!(nonce.len() >= 24);
        let enc_key = self.hasher.hash(|hasher| {
            hasher.update(key);
            hasher.update(DOMAIN_ENCRYPT);
            hasher.update(nonce);
        });

        let auth_key = self.hasher.hash(|hasher| {
            hasher.update(key);
            hasher.update(DOMAIN_AUTH);
            hasher.update(nonce);
        });
        assert_ne!(enc_key, auth_key);

        (enc_key.into(), auth_key.into())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::{OsRng, RngCore};
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn encrypt_decrypt_raw(msg: Vec<u8>, aad: Vec<u8>) {
        let mut aead = ChaChaBlake3::new();
        if msg.is_empty() { return; }
        let mut nonce = [0; TOTAL_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        let mut key = [0; 32];
        OsRng.fill_bytes(&mut key);

        let (encrypted, mac) = aead.encrypt_raw(
            msg.clone(),
            &nonce,
            &aad,
            &key,
        ).expect("encrypted failed");
        let decrypted = aead.decrypt_raw(
            encrypted.clone(),
            &mac,
            &nonce,
            &aad,
            &key,
        ).expect("decrypted failed");

        assert_ne!(encrypted, decrypted);
        assert_ne!(msg, encrypted);
        assert_eq!(msg, decrypted);
    }

    #[quickcheck]
    fn encrypt_decrypt(msg: Vec<u8>, aad: Vec<u8>) {
        let mut aead = ChaChaBlake3::new();
        if msg.is_empty() { return; }

        let mut key = [0; 32];
        OsRng.fill_bytes(&mut key);

        let encrypted = aead.encrypt(
            msg.clone(),
            &aad,
            &key,
        ).expect("encrypted: failed");

        let decrypted = aead.decrypt(
            encrypted.clone(),
            &key,
        ).expect("decrypted: failed");

        assert_ne!(encrypted.ciphertext, decrypted);
        assert_ne!(msg, encrypted.ciphertext);
        assert_eq!(msg, decrypted);
    }

    #[quickcheck]
    fn encrypt_decrypt_fail_aad(msg: Vec<u8>, aad: Vec<u8>) {
        let mut aead = ChaChaBlake3::new();
        if msg.is_empty() { return; }

        let mut key = [0; 32];
        OsRng.fill_bytes(&mut key);

        let mut encrypted = aead.encrypt(
            msg.clone(),
            &aad,
            &key,
        ).expect("encrypted: failed");

        // Adding a byte should break the aad
        encrypted.aad.push(123);

        let decrypted = aead.decrypt(
            encrypted.clone(),
            &key,
        ).expect_err("should have failed due to aad mutation");

        match decrypted {
            AeadError::AuthenticationError => (),
            otherwise => panic!("Failed: {:?}", otherwise)
        };
    }

    #[quickcheck]
    fn encrypt_decrypt_fail_cipher(msg: Vec<u8>, aad: Vec<u8>) {
        let mut aead = ChaChaBlake3::new();
        if msg.is_empty() { return; }

        let mut key = [0; 32];
        OsRng.fill_bytes(&mut key);

        let mut encrypted = aead.encrypt(
            msg.clone(),
            &aad,
            &key,
        ).expect("encrypted: failed");

        // Adding a byte should break the aad
        encrypted.ciphertext.push(123);

        let decrypted = aead.decrypt(
            encrypted.clone(),
            &key,
        ).expect_err("should have failed due to ciphertext mutation");

        match decrypted {
            AeadError::AuthenticationError => (),
            otherwise => panic!("Failed: {:?}", otherwise)
        };
    }
}
