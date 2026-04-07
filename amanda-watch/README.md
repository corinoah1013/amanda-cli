# amanda-watch

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

```bash
cargo build --release
```

El binario estará en `target/release/amanda-watch`.

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

# Combinar filtros
amanda-watch --filter-name "python" --filter-cpu-above 10
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

# Configuración desde archivo
amanda-watch --alert-config alerts.json
```

### Auditoría y Reportes

```bash
# Guardar snapshot en formato .amaudit
amanda-watch --snapshot system.amaudit

# Generar reporte .amrpt
amanda-watch --report report.amrpt --system

# Pipeline: procesos de alto consumo -> archivo
amanda-watch --format json --filter-cpu-above 50 | jq -r '.processes[].name' > high-cpu.txt
```

## Formatos Amanda OS

| Extensión | Descripción |
|-----------|-------------|
| `.amaudit` | Snapshot de estado del sistema con hash chain (auditable) |
| `.amrpt` | Reporte estructurado con métricas y tabla de procesos |

## Ejemplos de scripting

```bash
#!/bin/bash
# Monitorear uso de memoria de un servicio

SERVICE="postgres"
THRESHOLD_MB=500

amanda-watch --filter-name "$SERVICE" --format json | jq -e --arg MB "$THRESHOLD_MB" '
  .processes | map(select(.memory_bytes > ($MB | tonumber) * 1024 * 1024)) | length > 0
' && echo "⚠️ $SERVICE usando más de ${THRESHOLD_MB}MB"
```

## Licencia

MIT - Ver LICENSE para detalles.
