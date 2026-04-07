# amanda-watch

[![Crates.io](https://img.shields.io/crates/v/amanda-watch)](https://crates.io/crates/amanda-watch)
[![Documentation](https://docs.rs/amanda-watch/badge.svg)](https://docs.rs/amanda-watch)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Monitor de procesos y recursos del sistema para Amanda OS. Equivalente a `htop` pero diseñado para scripting, automatización y pipelines — salida estructurada, filtros programáticos, alertas configurables.

## Características

- **Monitoreo de procesos:** PID, nombre, CPU%, memoria, estado, usuario
- **Recursos del sistema:** CPU total, RAM, swap, load average
- **Salida estructurada:** JSON, CSV o texto formateado
- **Filtros:** por nombre (regex), PID, uso de CPU/memoria
- **Alertas:** umbrales configurables para CPU y memoria
- **Snapshots:** guardar estado del sistema en formato `.amaudit` para auditoría
- **Reportes:** exportar a formato `.amrpt` (Amanda Report)
- **Modo continuo:** polling configurable (`--interval`)

## Instalación

### Script automático

```bash
curl -fsSL https://raw.githubusercontent.com/corinoah1013/amanda-cli/main/install.sh | bash
```

### Homebrew

```bash
brew tap corinoah1013/amanda-cli
brew install amanda-watch
```

### Cargo

```bash
cargo install amanda-watch
```

### Desde código fuente

```bash
git clone https://github.com/corinoah1013/amanda-cli.git
cd amanda-cli
cargo build --release
sudo cp target/release/amanda-watch /usr/local/bin/
```

## Uso

### Básico

```bash
# Mostrar top 20 procesos por uso de CPU
amanda-watch

# Incluir información del sistema
amanda-watch --system

# Salida en JSON para pipelines
amanda-watch --format json | jq '.processes[] | select(.cpu_percent > 10)'

# Exportar a CSV
amanda-watch --format csv > processes.csv
```

### Filtros

```bash
# Filtrar por nombre (regex)
amanda-watch --filter-name "nginx|php-fpm"

# Filtrar por PID específico
amanda-watch --filter-pid 1234

# Procesos usando más del 50% CPU
amanda-watch --filter-cpu-above 50

# Procesos usando más de 100MB RAM
amanda-watch --filter-mem-above 100
```

### Monitoreo continuo

```bash
# Actualizar cada 5 segundos
amanda-watch --interval 5 --system

# Guardar snapshots periódicos
amanda-watch --interval 60 --snapshot /var/log/system.amaudit
```

### Alertas

```bash
# Alertar si CPU del sistema supera 80%
amanda-watch --alert-cpu 80

# Alertar si memoria supera 90%
amanda-watch --alert-mem 90

# Vigilar proceso específico
amanda-watch --watch-process nginx
```

### Auditoría y Reportes

```bash
# Guardar snapshot en formato .amaudit
amanda-watch --snapshot system.amaudit

# Generar reporte .amrpt
amanda-watch --report report.amrpt --system
```

## Formatos Amanda OS

| Extensión | Descripción |
|-----------|-------------|
| `.amaudit` | Snapshot de estado del sistema con hash chain (auditable) |
| `.amrpt` | Reporte estructurado con métricas y tabla de procesos |

## Licencia

MIT — Ver [LICENSE](../LICENSE).
