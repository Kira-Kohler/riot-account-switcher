use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const SETTINGS_FILE: &str = "RiotGamesPrivateSettings.yaml";

pub fn settings_path() -> Option<PathBuf> {
    let appdata = std::env::var("LOCALAPPDATA").ok()?;
    Some(
        PathBuf::from(appdata)
            .join("Riot Games")
            .join("Riot Client")
            .join("Data")
            .join(SETTINGS_FILE),
    )
}

pub fn find_client_exe() -> Option<PathBuf> {
    let pf   = std::env::var("PROGRAMFILES").unwrap_or_default();
    let pf86 = std::env::var("PROGRAMFILES(X86)").unwrap_or_default();

    let candidates = [
        r"C:\Riot Games\Riot Client\RiotClientServices.exe".to_string(),
        format!(r"{pf}\Riot Games\Riot Client\RiotClientServices.exe"),
        format!(r"{pf86}\Riot Games\Riot Client\RiotClientServices.exe"),
    ];

    candidates.into_iter().map(PathBuf::from).find(|p| p.exists())
}

pub fn read_tokens() -> Result<String> {
    let path = settings_path().context("LOCALAPPDATA not set")?;
    if !path.exists() {
        bail!(
            "Session file not found — make sure the Riot Client is running\nand you are logged in.\n\nExpected: {}",
            path.display()
        );
    }
    Ok(std::fs::read_to_string(&path)?)
}

pub fn write_tokens(data: &str) -> Result<()> {
    let path = settings_path().context("LOCALAPPDATA not set")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, data)?;
    Ok(())
}

pub fn fetch_riot_id_live() -> Option<String> {
    let lockfile = std::fs::read_to_string(lockfile_path()?).ok()?;
    let p: Vec<&str> = lockfile.trim().split(':').collect();
    if p.len() < 5 { return None; }
    let (port, password) = (p[2], p[3]);

    let connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build().ok()?;
    let agent = ureq::AgentBuilder::new()
        .tls_connector(Arc::new(connector))
        .timeout_connect(Duration::from_secs(2))
        .timeout(Duration::from_secs(3))
        .build();

    let auth = format!("Basic {}", B64.encode(format!("riot:{password}")));
    let url = format!("https://127.0.0.1:{port}/riot-client-auth/v1/userinfo");

    let text = agent
        .get(&url)
        .set("Authorization", &auth)
        .call().ok()?
        .into_string().ok()?;
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;

    let name = json["acct"]["game_name"].as_str()?;
    let tag  = json["acct"]["tag_line"].as_str().unwrap_or("");
    Some(if tag.is_empty() { name.to_string() } else { format!("{name}#{tag}") })
}

fn lockfile_path() -> Option<PathBuf> {
    let appdata = std::env::var("LOCALAPPDATA").ok()?;
    Some(PathBuf::from(appdata)
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile"))
}

pub fn extract_riot_id(data: &str) -> Option<String> {
    let mut game_name: Option<String> = None;
    let mut tag_line: Option<String> = None;

    for line in data.lines() {
        let t = line.trim();
        if game_name.is_none() {
            for prefix in &["gameName:", "game-name:"] {
                if let Some(v) = t.strip_prefix(prefix) {
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if !v.is_empty() && !v.starts_with('{') {
                        game_name = Some(v.to_string());
                        break;
                    }
                }
            }
        }
        if tag_line.is_none() {
            if let Some(v) = t.strip_prefix("tagLine:") {
                let v = v.trim().trim_matches('"').trim_matches('\'');
                if !v.is_empty() && !v.starts_with('{') {
                    tag_line = Some(v.to_string());
                }
            }
        }
    }

    match (game_name, tag_line) {
        (Some(name), Some(tag)) => Some(format!("{name}#{tag}")),
        (Some(name), None) => Some(name),
        _ => None,
    }
}

pub fn extract_username(data: &str) -> Option<String> {
    for line in data.lines() {
        let t = line.trim();
        for prefix in &["username:", "user-name:", "acct:"] {
            if let Some(v) = t.strip_prefix(prefix) {
                let v = v.trim().trim_matches('"').trim_matches('\'');
                if !v.is_empty() && !v.starts_with('{') {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

pub fn kill_client() {
    let targets = [
        "RiotClientServices.exe",
        "RiotClientUx.exe",
        "RiotClientUxRender.exe",
        "RiotClientHelper.exe",
        "RiotClientCrashHandler.exe",
        "RiotClientElectron.exe",
        "LeagueClient.exe",
        "LeagueClientUx.exe",
        "VALORANT.exe",
    ];
    for t in &targets {
        let _ = Command::new("taskkill").args(["/F", "/IM", t]).output();
    }
    thread::sleep(Duration::from_millis(1500));
    if let Some(p) = lockfile_path() {
        let _ = std::fs::remove_file(p);
    }
}

pub fn launch_client() -> Result<()> {
    let path = find_client_exe()
        .ok_or_else(|| anyhow::anyhow!("Riot Client not found — checked common install paths"))?;
    Command::new(&path).spawn()?;
    Ok(())
}

pub fn clear_riot_credentials() {
    let Ok(out) = Command::new("cmdkey").arg("/list").output() else { return };
    let text = String::from_utf8_lossy(&out.stdout);
    let targets: Vec<String> = text
        .lines()
        .filter(|l| l.to_lowercase().contains("riot"))
        .filter_map(|l| l.trim().strip_prefix("Target: ").map(|s| s.trim().to_string()))
        .collect();
    for t in targets {
        let _ = Command::new("cmdkey").arg(format!("/delete:{t}")).output();
    }
}

pub fn switch_to(token_data: &str) -> Result<()> {
    kill_client();
    clear_riot_credentials();
    write_tokens(token_data)?;
    launch_client()?;
    Ok(())
}

pub fn logout() -> Result<()> {
    kill_client();
    write_tokens("")?;
    launch_client()?;
    Ok(())
}
