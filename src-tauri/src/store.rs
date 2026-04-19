use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

use crate::license::LicenseAccount;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountStore {
    pub accounts: Vec<LicenseAccount>,
    pub last_selected_email: Option<String>,
}

fn store_path() -> PathBuf {
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(".seevideo").join("accounts.enc")
}

fn derive_local_key() -> [u8; 32] {
    let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "seevideo-local".to_string());
    let mut hasher = Sha256::new();
    hasher.update(b"SeeVideo-Local-Key-v1");
    hasher.update(host.as_bytes());
    let digest = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest[..32]);
    key
}

pub fn load_store() -> anyhow::Result<AccountStore> {
    let p = store_path();
    if !p.exists() {
        return Ok(AccountStore::default());
    }
    let blob = fs::read_to_string(&p)?;
    let mut parts = blob.split('.');
    let iv_b64 = parts.next().ok_or_else(|| anyhow::anyhow!("bad store format"))?;
    let ct_b64 = parts.next().ok_or_else(|| anyhow::anyhow!("bad store format"))?;
    let tag_b64 = parts.next().ok_or_else(|| anyhow::anyhow!("bad store format"))?;

    let iv = URL_SAFE_NO_PAD.decode(iv_b64.as_bytes())?;
    let ct = URL_SAFE_NO_PAD.decode(ct_b64.as_bytes())?;
    let tag = URL_SAFE_NO_PAD.decode(tag_b64.as_bytes())?;

    let key = derive_local_key();
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    let mut combined = ct;
    combined.extend_from_slice(&tag);
    let plain = cipher
        .decrypt(Nonce::from_slice(&iv), combined.as_ref())
        .map_err(|_| anyhow::anyhow!("decrypt store failed"))?;
    let st: AccountStore = serde_json::from_slice(&plain)?;
    Ok(st)
}

pub fn save_store(st: &AccountStore) -> anyhow::Result<()> {
    let p = store_path();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }

    let key = derive_local_key();
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    let mut iv = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut iv);

    let plain = serde_json::to_vec(st)?;
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&iv), plain.as_ref())
        .map_err(|_| anyhow::anyhow!("encrypt store failed"))?;
    if encrypted.len() < 16 {
        anyhow::bail!("encrypted blob too short")
    }
    let split = encrypted.len() - 16;
    let (ct, tag) = encrypted.split_at(split);

    let blob = format!(
        "{}.{}.{}",
        URL_SAFE_NO_PAD.encode(iv),
        URL_SAFE_NO_PAD.encode(ct),
        URL_SAFE_NO_PAD.encode(tag)
    );
    fs::write(p, blob)?;
    Ok(())
}

pub fn merge_accounts(new_accounts: &[LicenseAccount]) -> anyhow::Result<usize> {
    let mut st = load_store()?;
    let mut added = 0usize;
    for a in new_accounts {
        let exists = st
            .accounts
            .iter()
            .any(|x| x.email.eq_ignore_ascii_case(&a.email));
        if !exists {
            st.accounts.push(a.clone());
            added += 1;
        }
    }
    save_store(&st)?;
    Ok(added)
}

pub fn get_last_selected_email() -> anyhow::Result<Option<String>> {
    Ok(load_store()?.last_selected_email)
}

pub fn set_last_selected_email(email: Option<String>) -> anyhow::Result<()> {
    let mut st = load_store()?;
    st.last_selected_email = email
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    save_store(&st)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static STORE_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn sample_account(email: &str) -> LicenseAccount {
        LicenseAccount {
            email: email.to_string(),
            password: "pw".to_string(),
            status: "active".to_string(),
            registertime: "2026-04-19T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn saves_and_loads_last_selected_email() {
        let _guard = STORE_TEST_LOCK.lock().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let old_home = std::env::var_os("HOME");
        std::env::set_var("HOME", temp_home.path());

        save_store(&AccountStore {
            accounts: vec![sample_account("b@example.com")],
            last_selected_email: Some("b@example.com".to_string()),
        })
        .unwrap();

        let loaded = load_store().unwrap();
        assert_eq!(loaded.last_selected_email.as_deref(), Some("b@example.com"));

        match old_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn merge_accounts_preserves_existing_last_selected_email() {
        let _guard = STORE_TEST_LOCK.lock().unwrap();
        let temp_home = tempfile::tempdir().unwrap();
        let old_home = std::env::var_os("HOME");
        std::env::set_var("HOME", temp_home.path());

        save_store(&AccountStore {
            accounts: vec![sample_account("b@example.com")],
            last_selected_email: Some("b@example.com".to_string()),
        })
        .unwrap();

        let added = merge_accounts(&[sample_account("a@example.com")]).unwrap();
        assert_eq!(added, 1);

        let loaded = load_store().unwrap();
        assert_eq!(loaded.last_selected_email.as_deref(), Some("b@example.com"));
        assert_eq!(loaded.accounts.len(), 2);

        match old_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }
}
