<div align="center">

<pre style="font-family:monospace; text-align:center;">
██   ██  ██ ██████  ██   ██ ██       █████  ██████  ███████ 
██  ██  ███ ██   ██ ██   ██ ██      ██   ██ ██   ██ ██      
█████    ██ ██████  ███████ ██      ███████ ██████  ███████ 
██  ██   ██ ██   ██      ██ ██      ██   ██ ██   ██      ██ 
██   ██  ██ ██   ██      ██ ███████ ██   ██ ██████  ███████ 
                                                            
                                                            
</pre>

🌐 &nbsp;[English](../README.md) · [Français](README.fr.md) · [Português](README.pt.md)

<br/>

### Riot Account Switcher

**Comparte tu cuenta de Riot sin compartir tu contraseña — cambia entre múltiples cuentas al instante.**  
Sin navegador, sin editar archivos manualmente — solo pulsa `Enter`.

<br/>

[![Última versión](https://img.shields.io/github/v/release/Kira-Kohler/riot-account-switcher?style=for-the-badge&color=ff4655&label=Descargar)](https://github.com/Kira-Kohler/riot-account-switcher/releases/latest)&nbsp;
[![Windows](https://img.shields.io/badge/Windows-10%2F11-5fcfff?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/Kira-Kohler/riot-account-switcher/releases)&nbsp;
[![Rust](https://img.shields.io/badge/Hecho%20con-Rust-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)&nbsp;
[![Licencia: MIT](https://img.shields.io/badge/Licencia-MIT-9b9ba8?style=for-the-badge)](../LICENSE)

<br/>

> ⚡ Cambia de cuenta en ~2 segundos &nbsp;|&nbsp; 🔒 Exportaciones cifradas AES-256-GCM &nbsp;|&nbsp; 📦 Un solo `.exe` portable

<br/>

![Preview](Preview.png)

</div>

---

## Por qué

Gestionar múltiples cuentas de League of Legends, VALORANT o TFT implica cerrar sesión constantemente, esperar a que el cliente de Riot se recargue e introducir contraseñas. Esta herramienta hace que el cambio de cuenta sea **instantáneo** intercambiando directamente el archivo de sesión del cliente de Riot — sin necesidad de credenciales después del primer guardado.

---

## ✨ Características

| | Función | Descripción |
|---|---|---|
| ⚡ | **Cambio instantáneo** | Cierra el cliente de Riot, cambia la sesión y lo relanza en ~2s |
| 🎮 | **Detección de Riot ID** | Lee `gameName#TAG` en directo desde la API local del cliente de Riot |
| 🔒 | **Exportaciones cifradas** | Comparte cuentas mediante archivos `.riotacc` (AES-256-GCM, nombre oculto) |
| 💾 | **Almacenamiento local** | Las sesiones se guardan en SQLite — nunca se envían a ningún servidor |
| 🚪 | **Atajo de cierre de sesión** | Limpia la sesión y relanza el cliente de Riot a la pantalla de inicio de sesión |
| 📂 | **Diálogo de archivos nativo** | Integración con el Explorador de Windows para importar archivos `.riotacc` |
| 🛡️ | **Elevación UAC** | Solicita permisos de administrador automáticamente mediante manifiesto integrado |
| 📦 | **Sin dependencias** | Un solo `.exe` portable, sin instalador, sin dependencias |

---

## 🚀 Primeros pasos

### Descarga (recomendado)

1. Ve a [**Releases**](https://github.com/Kira-Kohler/riot-account-switcher/releases/latest)
2. Descarga `K1R4LABS-RiotAccSwitcher.exe`
3. Colócalo donde quieras
4. Ejecútalo (aparecerá un aviso de UAC — es lo esperado)

### Compilar desde el código fuente

```powershell
git clone https://github.com/Kira-Kohler/riot-account-switcher
cd riot-account-switcher
cargo build --release
```

> Requiere [Rust](https://rustup.rs) con la toolchain `x86_64-pc-windows-msvc`.  
> Salida: `target\release\K1R4LABS-RiotAccSwitcher.exe`

---

## 🎮 Uso

### Atajos de teclado

| Tecla | Acción |
|-------|--------|
| `↑` / `↓` | Navegar por la lista de cuentas |
| `Enter` | Cambiar a la cuenta seleccionada |
| `S` | Guardar la sesión actual del cliente de Riot |
| `U` | Actualizar tokens + Riot ID de la cuenta seleccionada |
| `R` | Renombrar la cuenta seleccionada |
| `D` | Eliminar la cuenta seleccionada |
| `E` | Exportar a un archivo `.riotacc` cifrado |
| `I` | Importar desde un archivo `.riotacc` |
| `L` | Cerrar sesión — limpia la sesión y muestra la pantalla de inicio de sesión |
| `Q` | Salir |

### Flujo típico

```
1. Inicia sesión en tu primera cuenta a través del cliente de Riot
2. Abre K1R4LABS-RiotAccSwitcher
3. Pulsa [S] y dale un nombre a la cuenta  →  ¡guardada!
4. Repite para cada cuenta
5. Selecciona cualquier cuenta y pulsa [Enter] para cambiar al instante
```

---

<details>
<summary>📁 Exportar / Importar (archivos .riotacc)</summary>

<br/>

Exportar una cuenta para compartirla o hacer una copia de seguridad:

1. Selecciona la cuenta y pulsa `[E]`
2. Introduce una contraseña para proteger el archivo
3. Se crea un archivo `.riotacc` junto al ejecutable

Importar en cualquier máquina:

1. Pulsa `[I]` — se abre el diálogo de archivos de Windows
2. Selecciona el archivo `.riotacc`
3. Introduce la contraseña

**Seguridad:** El nombre de la cuenta y los datos de sesión están cifrados dentro del archivo. El envoltorio JSON contiene únicamente `v`, `salt`, `nonce` y `ciphertext` — nada en texto plano.

```json
{
  "v": 2,
  "salt": "<base64 — 16 bytes aleatorios>",
  "nonce": "<base64 — 12 bytes aleatorios>",
  "ciphertext": "<base64 — AES-256-GCM cifrado {name, data}>"
}
```

Derivación de clave: **PBKDF2-HMAC-SHA256** · 100 000 iteraciones · salt aleatorio de 16 bytes

</details>

<details>
<summary>🏗️ Arquitectura</summary>

<br/>

```
src/
├── main.rs     Punto de entrada — abre la BD, lanza la TUI
├── ui.rs       TUI completa: layout, renderizado, gestión de entrada y estado
├── riot.rs     Integración con el cliente de Riot
│               ├── Lectura / escritura del YAML de sesión
│               ├── Consultas a la API local mediante lockfile (Riot ID)
│               └── Gestión de procesos (matar / lanzar)
├── db.rs       Persistencia SQLite mediante rusqlite
└── crypto.rs   Exportación/importación AES-256-GCM — nombre cifrado dentro del ciphertext

assets/
├── K1R4LABS.ico    Icono de la aplicación (integrado en el binario)
└── Preview.png     Captura de pantalla

build.rs        winresource — integra icono, manifiesto UAC y metadatos del archivo
```

**Ubicación del archivo de sesión:**
```
%LOCALAPPDATA%\Riot Games\Riot Client\Data\RiotGamesPrivateSettings.yaml
```

**Fuente del Riot ID:**
```
%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile
→ HTTPS GET https://127.0.0.1:{puerto}/riot-client-auth/v1/userinfo
```

</details>

---

## 🛡️ Seguridad

- Las sesiones se almacenan **solo localmente** — sin peticiones de red a servidores externos
- La única comunicación externa es con `127.0.0.1` (el propio cliente de Riot)
- Los exportados `.riotacc` ocultan el nombre de la cuenta — el archivo no revela nada sin la contraseña
- **No compartas** tu `accounts.db` — almacena todas las sesiones sin protección por contraseña
- **Comparte archivos `.riotacc` solo con personas de confianza** — otorgan acceso completo a esa cuenta

---

## 📋 Requisitos

- Windows 10 o Windows 11
- [Cliente de Riot Games](https://www.riotgames.com) instalado
- Privilegios de administrador (solicitados automáticamente)

---

## 🤝 Contribuir

Las issues y pull requests son bienvenidas.  
Para cambios importantes, abre primero una issue para discutir lo que te gustaría modificar.

---

## 📄 Licencia

[MIT](../LICENSE) © 2026 K1R4LABS
