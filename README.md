<div align="center">

<pre style="font-family:monospace; text-align:center;">
██   ██  ██ ██████  ██   ██ ██       █████  ██████  ███████ 
██  ██  ███ ██   ██ ██   ██ ██      ██   ██ ██   ██ ██      
█████    ██ ██████  ███████ ██      ███████ ██████  ███████ 
██  ██   ██ ██   ██      ██ ██      ██   ██ ██   ██      ██ 
██   ██  ██ ██   ██      ██ ███████ ██   ██ ██████  ███████ 
                                                            
                                                            
</pre>

🌐 &nbsp;[Español](assets/README.es.md) · [Français](assets/README.fr.md) · [Português](assets/README.pt.md)

<br/>

### Riot Account Switcher

**Share your Riot account without sharing your password — switch between multiple accounts instantly.**  
No browser, no manual file editing — just press `Enter`.

<br/>

[![Latest Release](https://img.shields.io/github/v/release/Kira-Kohler/riot-account-switcher?style=for-the-badge&color=ff4655&label=Download)](https://github.com/Kira-Kohler/riot-account-switcher/releases/latest)&nbsp;
[![Windows](https://img.shields.io/badge/Windows-10%2F11-5fcfff?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/Kira-Kohler/riot-account-switcher/releases)&nbsp;
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)&nbsp;
[![License: MIT](https://img.shields.io/badge/License-MIT-9b9ba8?style=for-the-badge)](LICENSE)

<br/>

> ⚡ Switches accounts in ~2 seconds &nbsp;|&nbsp; 🔒 AES-256-GCM encrypted exports &nbsp;|&nbsp; 📦 Single portable `.exe`

<br/>

![Preview](assets/Preview.png)

</div>

---

## Why

Managing multiple League of Legends, VALORANT, or TFT accounts means constantly logging out, waiting for the Riot Client to reload, and typing passwords. This tool makes account switching **instant** by directly swapping the Riot Client session file — no credentials needed after the first save.

---

## ✨ Features

| | Feature | Description |
|---|---|---|
| ⚡ | **Instant switching** | Kills the Riot Client, swaps the session, relaunches in ~2s |
| 🎮 | **Riot ID detection** | Reads `gameName#TAG` live from the Riot Client's local API |
| 🔒 | **Encrypted exports** | Share accounts via `.riotacc` files (AES-256-GCM, name hidden) |
| 💾 | **Local-only storage** | Sessions stored in SQLite — never sent anywhere |
| 🚪 | **Logout shortcut** | Clears session and relaunches Riot Client to the login screen |
| 📂 | **Native file dialog** | Windows Explorer integration for importing `.riotacc` files |
| 🛡️ | **UAC elevation** | Requests admin rights automatically via embedded manifest |
| 📦 | **Zero runtime deps** | Single portable `.exe`, no installer, no dependencies |

---

## 🚀 Getting Started

### Download (recommended)

1. Go to [**Releases**](https://github.com/Kira-Kohler/riot-account-switcher/releases/latest)
2. Download `K1R4LABS-RiotAccSwitcher.exe`
3. Place it anywhere
4. Run it (UAC prompt will appear — this is expected)

### Build from source

```powershell
git clone https://github.com/Kira-Kohler/riot-account-switcher
cd riot-account-switcher
cargo build --release
```

> Requires [Rust](https://rustup.rs) with the `x86_64-pc-windows-msvc` toolchain.  
> Output: `target\release\K1R4LABS-RiotAccSwitcher.exe`

---

## 🎮 Usage

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate the account list |
| `Enter` | Switch to the selected account |
| `S` | Save the current Riot Client session |
| `U` | Update tokens + Riot ID for the selected account |
| `R` | Rename the selected account |
| `D` | Delete the selected account |
| `E` | Export to an encrypted `.riotacc` file |
| `I` | Import from a `.riotacc` file |
| `L` | Logout — clears session and shows Riot login screen |
| `Q` | Quit |

### Typical workflow

```
1. Log into your first account via Riot Client
2. Open K1R4LABS-RiotAccSwitcher
3. Press [S] and give the account a name  →  saved!
4. Repeat for each account
5. Select any account and press [Enter] to switch instantly
```

---

<details>
<summary>📁 Export / Import (.riotacc files)</summary>

<br/>

Export an account to share it or back it up:

1. Select the account and press `[E]`
2. Enter a password to protect the file
3. A `.riotacc` file is created next to the executable

Import on any machine:

1. Press `[I]` — a Windows file dialog opens
2. Select the `.riotacc` file
3. Enter the password

**Security:** The account name and session data are both encrypted inside the file. The JSON wrapper contains only `v`, `salt`, `nonce`, and `ciphertext` — nothing in plaintext.

```json
{
  "v": 2,
  "salt": "<base64 — random 16 bytes>",
  "nonce": "<base64 — random 12 bytes>",
  "ciphertext": "<base64 — AES-256-GCM encrypted {name, data}>"
}
```

Key derivation: **PBKDF2-HMAC-SHA256** · 100 000 iterations · 16-byte random salt

</details>

<details>
<summary>🏗️ Architecture</summary>

<br/>

```
src/
├── main.rs     Entry point — opens DB, launches TUI
├── ui.rs       Full TUI: layout, rendering, input handling, app state
├── riot.rs     Riot Client integration
│               ├── Read / write session YAML
│               ├── Lockfile-based local API queries (Riot ID)
│               └── Process management (kill / launch)
├── db.rs       SQLite persistence via rusqlite
└── crypto.rs   AES-256-GCM export/import — name encrypted inside ciphertext

assets/
├── K1R4LABS.ico    Embedded application icon
└── Preview.png     Screenshot

build.rs        winresource — embeds icon, UAC manifest, file metadata
```

**Session file location:**
```
%LOCALAPPDATA%\Riot Games\Riot Client\Data\RiotGamesPrivateSettings.yaml
```

**Riot ID source:**
```
%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
→ HTTPS GET https://127.0.0.1:{port}/riot-client-auth/v1/userinfo
```

</details>

---

## 🛡️ Security

- Sessions are stored **locally only** — no network requests to external servers
- The only external communication is with `127.0.0.1` (the Riot Client itself)
- `.riotacc` exports hide the account name — the file reveals nothing without the password
- **Do not share** your `accounts.db` — it stores all sessions with no password protection
- **Only share `.riotacc` files with people you trust** — they grant full access to that account

---

## 📋 Requirements

- Windows 10 or Windows 11
- [Riot Games client](https://www.riotgames.com) installed
- Administrator privileges (requested automatically)

---

## 🤝 Contributing

Issues and pull requests are welcome.  
For major changes, open an issue first to discuss what you'd like to change.

---

## 📄 License

[MIT](LICENSE) © 2026 K1R4LABS
