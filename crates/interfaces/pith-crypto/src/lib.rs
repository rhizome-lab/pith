//! Cryptographic interfaces.

/// A cryptographic hash function.
pub trait Hash {
    /// The output size in bytes.
    const OUTPUT_SIZE: usize;

    /// Create a new hasher.
    fn new() -> Self;

    /// Update the hasher with data.
    fn update(&mut self, data: &[u8]);

    /// Finalize and return the hash.
    fn finalize(self) -> Vec<u8>;

    /// Hash data in one shot.
    fn hash(data: &[u8]) -> Vec<u8>
    where
        Self: Sized,
    {
        let mut hasher = Self::new();
        hasher.update(data);
        hasher.finalize()
    }
}

/// HMAC (Hash-based Message Authentication Code).
pub trait Hmac {
    /// Create a new HMAC with the given key.
    fn new(key: &[u8]) -> Self;

    /// Update with data.
    fn update(&mut self, data: &[u8]);

    /// Finalize and return the MAC.
    fn finalize(self) -> Vec<u8>;

    /// Verify a MAC.
    fn verify(self, expected: &[u8]) -> bool
    where
        Self: Sized,
    {
        let computed = self.finalize();
        constant_time_eq(&computed, expected)
    }
}

/// Symmetric encryption.
pub trait Cipher {
    /// The key size in bytes.
    const KEY_SIZE: usize;

    /// The nonce size in bytes.
    const NONCE_SIZE: usize;

    /// The authentication tag size in bytes.
    const TAG_SIZE: usize;

    /// Encrypt data with the given key and nonce.
    /// Returns ciphertext with appended authentication tag.
    fn encrypt(key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError>;

    /// Decrypt data with the given key and nonce.
    fn decrypt(key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError>;
}

/// Cryptographic signature scheme.
pub trait Signature {
    /// The public key size in bytes.
    const PUBLIC_KEY_SIZE: usize;

    /// The secret key size in bytes.
    const SECRET_KEY_SIZE: usize;

    /// The signature size in bytes.
    const SIGNATURE_SIZE: usize;

    /// Generate a new keypair.
    fn generate_keypair() -> (Vec<u8>, Vec<u8>);

    /// Sign a message.
    fn sign(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>, CryptoError>;

    /// Verify a signature.
    fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, CryptoError>;
}

/// Key derivation function.
pub trait Kdf {
    /// Derive a key from a password and salt.
    fn derive(password: &[u8], salt: &[u8], output_len: usize) -> Vec<u8>;
}

/// Cryptographic errors.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// Invalid key size.
    #[error("invalid key size")]
    InvalidKeySize,
    /// Invalid nonce size.
    #[error("invalid nonce size")]
    InvalidNonceSize,
    /// Authentication failed (decryption).
    #[error("authentication failed")]
    AuthenticationFailed,
    /// Invalid signature.
    #[error("invalid signature")]
    InvalidSignature,
    /// Other error.
    #[error("{0}")]
    Other(String),
}

/// Constant-time equality comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
