//! Native implementation of pith-crypto using RustCrypto.

use rhizome_pith_crypto::{Cipher, CryptoError, Hash, Hmac, Kdf, Signature};

// ============================================================================
// Hashing
// ============================================================================

/// SHA-256 hash.
pub struct Sha256(sha2::Sha256);

impl Hash for Sha256 {
    const OUTPUT_SIZE: usize = 32;

    fn new() -> Self {
        use sha2::Digest;
        Self(sha2::Sha256::new())
    }

    fn update(&mut self, data: &[u8]) {
        use sha2::Digest;
        self.0.update(data);
    }

    fn finalize(self) -> Vec<u8> {
        use sha2::Digest;
        self.0.finalize().to_vec()
    }
}

/// SHA-512 hash.
pub struct Sha512(sha2::Sha512);

impl Hash for Sha512 {
    const OUTPUT_SIZE: usize = 64;

    fn new() -> Self {
        use sha2::Digest;
        Self(sha2::Sha512::new())
    }

    fn update(&mut self, data: &[u8]) {
        use sha2::Digest;
        self.0.update(data);
    }

    fn finalize(self) -> Vec<u8> {
        use sha2::Digest;
        self.0.finalize().to_vec()
    }
}

// ============================================================================
// HMAC
// ============================================================================

/// HMAC-SHA256.
pub struct HmacSha256(hmac::Hmac<sha2::Sha256>);

impl Hmac for HmacSha256 {
    fn new(key: &[u8]) -> Self {
        use hmac::Mac;
        Self(hmac::Hmac::new_from_slice(key).expect("HMAC can take any size key"))
    }

    fn update(&mut self, data: &[u8]) {
        use hmac::Mac;
        self.0.update(data);
    }

    fn finalize(self) -> Vec<u8> {
        use hmac::Mac;
        self.0.finalize().into_bytes().to_vec()
    }
}

// ============================================================================
// Symmetric Encryption
// ============================================================================

/// AES-256-GCM.
pub struct Aes256Gcm;

impl Cipher for Aes256Gcm {
    const KEY_SIZE: usize = 32;
    const NONCE_SIZE: usize = 12;
    const TAG_SIZE: usize = 16;

    fn encrypt(key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use aes_gcm::{aead::Aead, Aes256Gcm as AesGcm, KeyInit, Nonce};

        if key.len() != Self::KEY_SIZE {
            return Err(CryptoError::InvalidKeySize);
        }
        if nonce.len() != Self::NONCE_SIZE {
            return Err(CryptoError::InvalidNonceSize);
        }

        let cipher = AesGcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKeySize)?;
        let nonce = Nonce::from_slice(nonce);

        // For AAD, we'd need to use encrypt_in_place_detached or similar
        // Simplified version without AAD support for now
        let _ = aad; // TODO: support AAD

        cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| CryptoError::AuthenticationFailed)
    }

    fn decrypt(key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use aes_gcm::{aead::Aead, Aes256Gcm as AesGcm, KeyInit, Nonce};

        if key.len() != Self::KEY_SIZE {
            return Err(CryptoError::InvalidKeySize);
        }
        if nonce.len() != Self::NONCE_SIZE {
            return Err(CryptoError::InvalidNonceSize);
        }

        let cipher = AesGcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKeySize)?;
        let nonce = Nonce::from_slice(nonce);

        let _ = aad; // TODO: support AAD

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::AuthenticationFailed)
    }
}

/// ChaCha20-Poly1305.
pub struct ChaCha20Poly1305;

impl Cipher for ChaCha20Poly1305 {
    const KEY_SIZE: usize = 32;
    const NONCE_SIZE: usize = 12;
    const TAG_SIZE: usize = 16;

    fn encrypt(key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use chacha20poly1305::{aead::Aead, ChaCha20Poly1305 as ChaCha, KeyInit, Nonce};

        if key.len() != Self::KEY_SIZE {
            return Err(CryptoError::InvalidKeySize);
        }
        if nonce.len() != Self::NONCE_SIZE {
            return Err(CryptoError::InvalidNonceSize);
        }

        let cipher = ChaCha::new_from_slice(key).map_err(|_| CryptoError::InvalidKeySize)?;
        let nonce = Nonce::from_slice(nonce);

        let _ = aad; // TODO: support AAD

        cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| CryptoError::AuthenticationFailed)
    }

    fn decrypt(key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use chacha20poly1305::{aead::Aead, ChaCha20Poly1305 as ChaCha, KeyInit, Nonce};

        if key.len() != Self::KEY_SIZE {
            return Err(CryptoError::InvalidKeySize);
        }
        if nonce.len() != Self::NONCE_SIZE {
            return Err(CryptoError::InvalidNonceSize);
        }

        let cipher = ChaCha::new_from_slice(key).map_err(|_| CryptoError::InvalidKeySize)?;
        let nonce = Nonce::from_slice(nonce);

        let _ = aad; // TODO: support AAD

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::AuthenticationFailed)
    }
}

// ============================================================================
// Signatures
// ============================================================================

/// Ed25519 signatures.
pub struct Ed25519;

impl Signature for Ed25519 {
    const PUBLIC_KEY_SIZE: usize = 32;
    const SECRET_KEY_SIZE: usize = 32;
    const SIGNATURE_SIZE: usize = 64;

    fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        (
            verifying_key.to_bytes().to_vec(),
            signing_key.to_bytes().to_vec(),
        )
    }

    fn sign(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        use ed25519_dalek::{Signer, SigningKey};

        let secret_bytes: [u8; 32] = secret_key
            .try_into()
            .map_err(|_| CryptoError::InvalidKeySize)?;
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let signature = signing_key.sign(message);

        Ok(signature.to_bytes().to_vec())
    }

    fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        use ed25519_dalek::{Signature as EdSig, Verifier, VerifyingKey};

        let public_bytes: [u8; 32] = public_key
            .try_into()
            .map_err(|_| CryptoError::InvalidKeySize)?;
        let verifying_key =
            VerifyingKey::from_bytes(&public_bytes).map_err(|_| CryptoError::InvalidKeySize)?;

        let sig_bytes: [u8; 64] = signature
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?;
        let sig = EdSig::from_bytes(&sig_bytes);

        Ok(verifying_key.verify(message, &sig).is_ok())
    }
}

// ============================================================================
// Key Derivation
// ============================================================================

/// Argon2id key derivation.
pub struct Argon2id;

impl Kdf for Argon2id {
    fn derive(password: &[u8], salt: &[u8], output_len: usize) -> Vec<u8> {
        use argon2::Argon2;

        let mut output = vec![0u8; output_len];
        Argon2::default()
            .hash_password_into(password, salt, &mut output)
            .expect("Argon2 derivation failed");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_works() {
        let hash = Sha256::hash(b"hello");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn hmac_sha256_works() {
        let mut mac = HmacSha256::new(b"secret");
        mac.update(b"message");
        let result = mac.finalize();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn aes_gcm_roundtrip() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"hello world";

        let ciphertext = Aes256Gcm::encrypt(&key, &nonce, plaintext, &[]).unwrap();
        let decrypted = Aes256Gcm::decrypt(&key, &nonce, &ciphertext, &[]).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn chacha_roundtrip() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let plaintext = b"hello world";

        let ciphertext = ChaCha20Poly1305::encrypt(&key, &nonce, plaintext, &[]).unwrap();
        let decrypted = ChaCha20Poly1305::decrypt(&key, &nonce, &ciphertext, &[]).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn ed25519_sign_verify() {
        let (public_key, secret_key) = Ed25519::generate_keypair();
        let message = b"hello world";

        let signature = Ed25519::sign(&secret_key, message).unwrap();
        let valid = Ed25519::verify(&public_key, message, &signature).unwrap();

        assert!(valid);
    }

    #[test]
    fn argon2_derives() {
        let password = b"password";
        let salt = b"saltsalt"; // Argon2 needs at least 8 bytes
        let derived = Argon2id::derive(password, salt, 32);
        assert_eq!(derived.len(), 32);
    }
}
