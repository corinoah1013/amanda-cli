# Changelog

Todos los cambios notables de este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [Unreleased]

### Added
- Nuevas funcionalidades en desarrollo

## [0.1.0] - 2026-04-07

### Added
- **amanda-watch v0.1.0**: Monitor de procesos y recursos del sistema
  - Monitoreo en tiempo real de procesos (PID, nombre, CPU%, memoria, estado, usuario)
  - Información de recursos del sistema (CPU, RAM, swap, load average)
  - Tres formatos de salida: texto formateado, JSON, CSV
  - Filtros: por nombre (regex), PID, uso de CPU/memoria
  - Alertas configurables: umbrales de CPU/memoria, watch de procesos
  - Snapshots en formato `.amaudit` con hash chain (tamper-evident)
  - Reportes estructurados en formato `.amrpt`
  - Modo continuo con polling configurable
  
- **amanda-core v0.1.0**: Biblioteca compartida
  - Formato `.amaudit`: Audit trail inmutable con hash chain SHA-256
  - Formato `.amrpt`: Reportes estructurados con secciones
  - Formato `.amconf`: Configuración multi-perfil XDG-compliant
  - Tipos compartidos y utilidades

### Infrastructure
- GitHub Actions: CI/CD con tests, clippy, builds multi-plataforma
- Script de instalación automática (`install.sh`)
- Builder de paquetes Debian (`.deb`)
- Soporte para Linux x64/ARM64 y macOS x64/ARM64

[Unreleased]: https://github.com/corinoah1013/amanda-cli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/corinoah1013/amanda-cli/releases/tag/v0.1.0
