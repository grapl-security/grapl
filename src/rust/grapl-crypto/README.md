## grapl-crypto

There are two main ways to encrypt/decrypt your data using this library.

1. Using a symmetric key
2. Using an asymmetric key pair

In general you should choose (1) when the service performing the encryption is the same as the service performing the decryption,
and you should choose (2) when the service performing the encryption is different from the one performing decryption.

When in doubt (2) is strictly more flexible than (1), so go with that.

The asymmetric implementation is really just a wrapper over the symmetric implementation with the additional work of performing
a symmetric key derivation using x25519.

The tl;dr is that you get ChaCha12Blake3 as your encryption algorithm and x25519 as your key derivation algorithm.

This crate is suitable if your key is already cryptographically secure ie: provisioned via Vault. If you are encrypting
your data with a password this crate is *not* suitable unless you have already prepared the key with a key derivation algorithm.

If you need to prepare a key you can use the argon2 crate.
