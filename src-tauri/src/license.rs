use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rsa::{Oaep, RsaPrivateKey, pkcs1::DecodeRsaPrivateKey, pkcs8::DecodePrivateKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Deserialize)]
pub struct Envelope {
    pub v: u8,
    pub alg: String,
    pub iv: String,
    pub tag: String,
    pub ek: String,
    pub ct: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    pub v: u8,
    pub license_id: String,
    pub exp: String,
    pub created_at: String,
    pub accounts: Vec<LicenseAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseAccount {
    pub email: String,
    pub password: String,
    pub status: String,
    pub registertime: String,
}

pub fn decode_activation_code(code: &str, private_pem: &str) -> anyhow::Result<LicensePayload> {
    let code = code.trim();
    let raw = URL_SAFE_NO_PAD.decode(code.as_bytes())?;
    let env: Envelope = serde_json::from_slice(&raw)?;

    if env.v != 1 {
        anyhow::bail!("unsupported activation code version")
    }

    let iv = URL_SAFE_NO_PAD.decode(env.iv.as_bytes())?;
    let tag = URL_SAFE_NO_PAD.decode(env.tag.as_bytes())?;
    let ek = URL_SAFE_NO_PAD.decode(env.ek.as_bytes())?;
    let ct = URL_SAFE_NO_PAD.decode(env.ct.as_bytes())?;

    let private_key = RsaPrivateKey::from_pkcs8_pem(private_pem)
        .or_else(|_| RsaPrivateKey::from_pkcs1_pem(private_pem))?;

    let dek = private_key.decrypt(Oaep::new::<Sha256>(), &ek)?;
    if dek.len() != 32 {
        anyhow::bail!("invalid data key length")
    }

    let cipher = Aes256Gcm::new_from_slice(&dek)?;
    let mut combined = ct.clone();
    combined.extend_from_slice(&tag);

    let plaintext = cipher
        .decrypt(Nonce::from_slice(&iv), combined.as_ref())
        .map_err(|_| anyhow::anyhow!("decrypt payload failed"))?;
    let payload: LicensePayload = serde_json::from_slice(&plaintext)?;
    Ok(payload)
}
