use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use pbkdf2::pbkdf2_hmac;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::PathBuf;

const PBKDF2_ITERS: u32 = 100_000;

#[derive(Serialize, Deserialize)]
struct ExportFileV2 {
    v: u32,
    salt: String,
    nonce: String,
    ciphertext: String,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    name: String,
    data: String,
    #[serde(default)]
    riot_id: Option<String>,
}

pub fn export(name: &str, token_data: &str, riot_id: Option<&str>, password: &str) -> Result<String> {
    let mut salt = [0u8; 16];
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let plaintext = serde_json::to_vec(&Payload {
        name: name.to_string(),
        data: token_data.to_string(),
        riot_id: riot_id.map(|s| s.to_string()),
    })?;

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_slice())
        .map_err(|_| anyhow!("Encryption failed"))?;

    Ok(serde_json::to_string_pretty(&ExportFileV2 {
        v: 2,
        salt: B64.encode(salt),
        nonce: B64.encode(nonce_bytes),
        ciphertext: B64.encode(ct),
    })?)
}

pub fn import(json: &str, password: &str) -> Result<(String, String, Option<String>)> {
    let file: ExportFileV2 = serde_json::from_str(json)
        .map_err(|_| anyhow!("Invalid .riotacc file format"))?;
    let payload = decrypt_bytes(&file.salt, &file.nonce, &file.ciphertext, password)?;
    let p: Payload = serde_json::from_slice(&payload)
        .map_err(|_| anyhow!("Corrupted file contents"))?;
    Ok((p.name, p.data, p.riot_id))
}

fn decrypt_bytes(salt_b64: &str, nonce_b64: &str, ct_b64: &str, password: &str) -> Result<Vec<u8>> {
    let salt  = B64.decode(salt_b64)?;
    let nonce = B64.decode(nonce_b64)?;
    let ct    = B64.decode(ct_b64)?;
    let key   = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    cipher
        .decrypt(Nonce::from_slice(&nonce), ct.as_slice())
        .map_err(|_| anyhow!("Wrong password or corrupted file"))
}

pub fn export_path(name: &str) -> PathBuf {
    let safe: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    exe_dir().join(format!("{safe}.riotacc"))
}

pub fn resolve_import_path(input: &str) -> PathBuf {
    let p = PathBuf::from(input);
    if p.is_absolute() { return p; }
    let candidate = exe_dir().join(input);
    if candidate.exists() { candidate } else { p }
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERS, &mut key);
    key
}
