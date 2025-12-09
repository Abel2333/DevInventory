use anyhow::Result;
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, aead::Aead, aead::KeyInit};
use rand::RngCore;
use zeroize::Zeroize;

#[derive(Clone)]
pub struct MasterKey(pub(crate) [u8; 32]);

impl Zeroize for MasterKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for MasterKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

pub struct SecretCrypto {
    key: MasterKey,
}

impl SecretCrypto {
    pub fn new(key: MasterKey) -> Self {
        Self { key }
    }

    pub fn encrypt(&self, aad_label: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        let mut rng = rand::rng();
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key.0));
        let mut aad = aad_label.as_bytes().to_vec();
        let mut ciphertext = cipher
            .encrypt(
                nonce,
                chacha20poly1305::aead::Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|e| anyhow::anyhow!(format!("encrypt failed: {e:?}")))?;
        // store nonce || ciphertext
        let mut out = Vec::with_capacity(12 + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.append(&mut ciphertext);
        aad.zeroize();
        Ok(out)
    }

    pub fn decrypt(&self, aad_label: &str, blob: &[u8]) -> Result<Vec<u8>> {
        if blob.len() < 12 {
            return Err(anyhow::anyhow!("ciphertext too short"));
        }
        let (nonce_bytes, ct) = blob.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key.0));
        let plaintext = cipher
            .decrypt(
                nonce,
                chacha20poly1305::aead::Payload {
                    msg: ct,
                    aad: aad_label.as_bytes(),
                },
            )
            .map_err(|e| anyhow::anyhow!(format!("decrypt failed: {e:?}")))?;
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = MasterKey([7u8; 32]);
        let crypto = SecretCrypto::new(key.clone());
        let plaintext = b"hello-secret";
        let ct = crypto.encrypt("name", plaintext).expect("encrypt");
        assert_ne!(ct, plaintext);
        let pt = crypto.decrypt("name", &ct).expect("decrypt");
        assert_eq!(pt, plaintext);
    }
}
