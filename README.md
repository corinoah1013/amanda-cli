# Amanda OS CLI Tools

[![CI](https://github.com/corinoah1013/amanda-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/corinoah1013/amanda-cli/actions/workflows/ci.yml)
[![Release](https://github.com/corinoah1013/amanda-cli/actions/workflows/release.yml/badge.svg)](https://github.com/corinoah1013/amanda-cli/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)

Herramientas CLI públicas del ecosistema Amanda OS — componentes reales del sistema que demuestran capacidad técnica en Rust para operaciones de sistemas, networking y protocolos.

> "El proyecto principal no puede mostrarse, pero estas herramientas salen directamente de él."

## Herramientas

| Herramienta | Stack Rust | Propósito | Estado |
|-------------|-----------|-----------|--------|
| [`amanda-watch`](./amanda-watch) | sysinfo, tokio, serde | Monitoreo de procesos y recursos | ✅ v0.1.0 |
| `amanda-logs` | sha2, serde, regex, tokio | Análisis de logs y audit trail inmutable | 🔜 |
| `amanda-mail` | async-imap, rustls, tokio | Cliente IMAP para automatización | 🔜 |

## Instalación Rápida

### Opción 1: Script de instalación automática (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/corinoah1013/amanda-cli/main/install.sh | bash
```

### Opción 2: Homebrew (macOS/Linux)

```bash
brew tap corinoah1013/amanda-cli
brew install amanda-watch
```

### Opción 3: Descargar binario pre-compilado

```bash
# Linux x64
wget https://github.com/corinoah1013/amanda-cli/releases/latest/download/amanda-watch-linux-x64.tar.gz
tar xzf amanda-watch-linux-x64.tar.gz
sudo mv amanda-watch /usr/local/bin/

# macOS (Intel)
wget https://github.com/corinoah1013/amanda-cli/releases/latest/download/amanda-watch-macos-x64.tar.gz
tar xzf amanda-watch-macos-x64.tar.gz
sudo mv amanda-watch /usr/local/bin/

# macOS (Apple Silicon)
wget https://github.com/corinoah1013/amanda-cli/releases/latest/download/amanda-watch-macos-arm64.tar.gz
tar xzf amanda-watch-macos-arm64.tar.gz
sudo mv amanda-watch /usr/local/bin/
```

### Opción 4: Debian/Ubuntu (.deb)

```bash
wget https://github.com/corinoah1013/amanda-cli/releases/latest/download/amanda-watch_0.1.0_amd64.deb
sudo dpkg -i amanda-watch_0.1.0_amd64.deb
```

### Opción 5: Compilar desde fuente

```bash
git clone https://github.com/corinoah1013/amanda-cli.git
cd amanda-cli
cargo build --release
sudo cp target/release/amanda-watch /usr/local/bin/
```

### Opción 6: Via cargo

```bash
cargo install amanda-watch
```

## Uso Rápido

```bash
# Ver top 20 procesos
amanda-watch

# Con información del sistema
amanda-watch --system

# Salida JSON para pipelines
amanda-watch --format json | jq '.processes[] | select(.cpu_percent > 10)'

# Guardar snapshot de auditoría
amanda-watch --snapshot system.amaudit

# Monitoreo continuo cada 5 segundos
amanda-watch --interval 5 --system
```

## Estructura del Proyecto

```
.
├── Cargo.toml          # Workspace configuration
├── amanda-core/        # Biblioteca compartida
│   ├── amaudit.rs      # Hash chain audit format (.amaudit)
│   ├── amrpt.rs        # Structured report format (.amrpt)
│   └── amconf.rs       # Configuration format (.amconf)
├── amanda-watch/       # Process monitor
├── amanda-logs/        # Log analyzer (TODO)
└── amanda-mail/        # IMAP client (TODO)
```

## Formatos Amanda OS

Todas las herramientas utilizan formatos nativos del ecosistema:

| Extensión | Propósito |
|-----------|-----------|
| `.amaudit` | Audit trail inmutable con hash chain (tamper-evident) |
| `.amrpt` | Reportes estructurados exportables |
| `.amconf` | Configuración multi-perfil |

## Desarrollo

```bash
# Compilar
cargo build --release

# Tests
cargo test --workspace

# Formato
cargo fmt --all

# Linting
cargo clippy --all-targets --all-features

# Crear paquete .deb (requiere cargo-deb o script)
./scripts/build-deb.sh
```

## CI/CD

- **GitHub Actions** compila automáticamente para:
  - Linux x64
  - Linux ARM64
  - macOS x64 (Intel)
  - macOS ARM64 (Apple Silicon)
- **Releases automáticos** al crear un tag `v*`
- **Publicación a crates.io** (opcional)

## Contribuir

¡Las contribuciones son bienvenidas! Por favor lee nuestra [Guía de Contribución](CONTRIBUTING.md) y el [Código de Conducta](CODE_OF_CONDUCT.md).

### Reportar bugs o solicitar features

- [Bug Report](https://github.com/corinoah1013/amanda-cli/issues/new?template=bug_report.md)
- [Feature Request](https://github.com/corinoah1013/amanda-cli/issues/new?template=feature_request.md)

## Roadmap

- [x] amanda-watch v0.1.0
- [ ] amanda-watch v0.2.0 (UI improvements)
- [ ] amanda-logs v0.1.0
- [ ] amanda-mail v0.1.0

## Narrativa

Estas herramientas son los componentes CLI públicos de Amanda OS, un sistema operativo completo cuya arquitectura core (kernel Rust 64/128-bit) es privada. Cada herramienta:

1. Resuelve un problema real de sistemas
2. Demuestra capacidad técnica en Rust idiomático
3. Está alineada con el stack de Mozilla/Thunderbird (migración Rust)
4. Produce artefactos útiles para sysadmins y DevOps

## Licencia

MIT — Ver [LICENSE](LICENSE).

## Contacto

- GitHub Issues: [github.com/corinoah1013/amanda-cli/issues](https://github.com/corinoah1013/amanda-cli/issues)
- Email: corinoah1013@github.com

---

<p align="center">Parte del ecosistema <strong>Amanda OS</strong></p>
