<div align="center">

<pre style="font-family:monospace; text-align:center;">
██   ██  ██ ██████  ██   ██ ██       █████  ██████  ███████ 
██  ██  ███ ██   ██ ██   ██ ██      ██   ██ ██   ██ ██      
█████    ██ ██████  ███████ ██      ███████ ██████  ███████ 
██  ██   ██ ██   ██      ██ ██      ██   ██ ██   ██      ██ 
██   ██  ██ ██   ██      ██ ███████ ██   ██ ██████  ███████ 
                                                            
                                                            
</pre>

🌐 &nbsp;[English](../README.md) · [Español](README.es.md) · [Français](README.fr.md)

<br/>

### Riot Account Switcher

**Compartilhe sua conta Riot sem compartilhar sua senha — troque entre múltiplas contas instantaneamente.**  
Sem navegador, sem edição manual de arquivos — apenas pressione `Enter`.

<br/>

[![Última versión](https://img.shields.io/github/v/release/Kira-Kohler/riot-account-switcher?style=for-the-badge&color=ff4655&label=Baixar)](https://github.com/Kira-Kohler/riot-account-switcher/releases/latest)&nbsp;
[![Windows](https://img.shields.io/badge/Windows-10%2F11-5fcfff?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/Kira-Kohler/riot-account-switcher/releases)&nbsp;
[![Rust](https://img.shields.io/badge/Feito%20com-Rust-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)&nbsp;
[![Licença: MIT](https://img.shields.io/badge/Licença-MIT-9b9ba8?style=for-the-badge)](../LICENSE)

<br/>

> ⚡ Troca de conta em ~2 segundos &nbsp;|&nbsp; 🔒 Exportações cifradas AES-256-GCM &nbsp;|&nbsp; 📦 Um único `.exe` portátil

<br/>

![Preview](Preview.png)

</div>

---

## Por quê

Gerenciar múltiplas contas de League of Legends, VALORANT ou TFT significa deslogar constantemente, esperar o cliente Riot recarregar e digitar senhas. Esta ferramenta torna a troca de contas **instantânea** ao trocar diretamente o arquivo de sessão do cliente Riot — sem necessidade de credenciais após o primeiro salvamento.

---

## ✨ Funcionalidades

| | Função | Descrição |
|---|---|---|
| ⚡ | **Troca instantânea** | Fecha o cliente Riot, troca a sessão e relança em ~2s |
| 🎮 | **Detecção do Riot ID** | Lê `gameName#TAG` ao vivo da API local do cliente Riot |
| 🔒 | **Exportações cifradas** | Compartilhe contas via arquivos `.riotacc` (AES-256-GCM, nome oculto) |
| 💾 | **Armazenamento local** | Sessões armazenadas em SQLite — nunca enviadas a nenhum servidor |
| 🚪 | **Atalho de logout** | Limpa a sessão e relança o cliente Riot na tela de login |
| 📂 | **Diálogo de arquivo nativo** | Integração com o Explorador do Windows para importar arquivos `.riotacc` |
| 🛡️ | **Elevação UAC** | Solicita privilégios de administrador automaticamente via manifesto integrado |
| 📦 | **Zero dependências** | Um único `.exe` portátil, sem instalador, sem dependências |

---

## 🚀 Primeiros Passos

### Download (recomendado)

1. Acesse [**Releases**](https://github.com/Kira-Kohler/riot-account-switcher/releases/latest)
2. Baixe `K1R4LABS-RiotAccSwitcher.exe`
3. Coloque-o em qualquer lugar
4. Execute-o (um aviso UAC aparecerá — isso é esperado)

### Compilar a partir do código-fonte

```powershell
git clone https://github.com/Kira-Kohler/riot-account-switcher
cd riot-account-switcher
cargo build --release
```

> Requer [Rust](https://rustup.rs) com a toolchain `x86_64-pc-windows-msvc`.  
> Saída: `target\release\K1R4LABS-RiotAccSwitcher.exe`

---

## 🎮 Uso

### Atalhos de teclado

| Tecla | Ação |
|-------|------|
| `↑` / `↓` | Navegar pela lista de contas |
| `Enter` | Trocar para a conta selecionada |
| `S` | Salvar a sessão atual do cliente Riot |
| `U` | Atualizar tokens + Riot ID da conta selecionada |
| `R` | Renomear a conta selecionada |
| `D` | Excluir a conta selecionada |
| `E` | Exportar para um arquivo `.riotacc` cifrado |
| `I` | Importar de um arquivo `.riotacc` |
| `L` | Logout — limpa a sessão e exibe a tela de login |
| `Q` | Sair |

### Fluxo típico

```
1. Faça login na sua primeira conta pelo cliente Riot
2. Abra o K1R4LABS-RiotAccSwitcher
3. Pressione [S] e dê um nome à conta  →  salvo!
4. Repita para cada conta
5. Selecione qualquer conta e pressione [Enter] para trocar instantaneamente
```

---

<details>
<summary>📁 Exportar / Importar (arquivos .riotacc)</summary>

<br/>

Exportar uma conta para compartilhá-la ou fazer backup:

1. Selecione a conta e pressione `[E]`
2. Digite uma senha para proteger o arquivo
3. Um arquivo `.riotacc` é criado ao lado do executável

Importar em qualquer máquina:

1. Pressione `[I]` — um diálogo de arquivo do Windows se abre
2. Selecione o arquivo `.riotacc`
3. Digite a senha

**Segurança:** O nome da conta e os dados de sessão estão ambos cifrados dentro do arquivo. O wrapper JSON contém apenas `v`, `salt`, `nonce` e `ciphertext` — nada em texto simples.

```json
{
  "v": 2,
  "salt": "<base64 — 16 bytes aleatórios>",
  "nonce": "<base64 — 12 bytes aleatórios>",
  "ciphertext": "<base64 — AES-256-GCM cifrado {name, data}>"
}
```

Derivação de chave: **PBKDF2-HMAC-SHA256** · 100 000 iterações · salt aleatório de 16 bytes

</details>

<details>
<summary>🏗️ Arquitetura</summary>

<br/>

```
src/
├── main.rs     Ponto de entrada — abre o BD, inicia a TUI
├── ui.rs       TUI completa: layout, renderização, tratamento de entrada e estado
├── riot.rs     Integração com o cliente Riot
│               ├── Leitura / escrita do YAML de sessão
│               ├── Consultas à API local via lockfile (Riot ID)
│               └── Gerenciamento de processos (matar / iniciar)
├── db.rs       Persistência SQLite via rusqlite
└── crypto.rs   Exportação/importação AES-256-GCM — nome cifrado dentro do ciphertext

assets/
├── K1R4LABS.ico    Ícone da aplicação (integrado no binário)
└── Preview.png     Captura de tela

build.rs        winresource — integra ícone, manifesto UAC e metadados do arquivo
```

**Localização do arquivo de sessão:**
```
%LOCALAPPDATA%\Riot Games\Riot Client\Data\RiotGamesPrivateSettings.yaml
```

**Fonte do Riot ID:**
```
%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
→ HTTPS GET https://127.0.0.1:{porta}/riot-client-auth/v1/userinfo
```

</details>

---

## 🛡️ Segurança

- As sessões são armazenadas **apenas localmente** — sem requisições de rede para servidores externos
- A única comunicação externa é com `127.0.0.1` (o próprio cliente Riot)
- Os exports `.riotacc` ocultam o nome da conta — o arquivo não revela nada sem a senha
- **Não compartilhe** seu `accounts.db` — ele armazena todas as sessões sem proteção por senha
- **Compartilhe arquivos `.riotacc` apenas com pessoas de confiança** — eles concedem acesso completo a essa conta

---

## 📋 Requisitos

- Windows 10 ou Windows 11
- [Cliente Riot Games](https://www.riotgames.com) instalado
- Privilégios de administrador (solicitados automaticamente)

---

## 🤝 Contribuindo

Issues e pull requests são bem-vindos.  
Para mudanças importantes, abra uma issue primeiro para discutir o que você gostaria de alterar.

---

## 📄 Licença

[MIT](../LICENSE) © 2026 K1R4LABS
