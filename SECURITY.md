# Política de Seguridad

## Versiones Soportadas

| Versión | Soportada          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reportar Vulnerabilidades

La seguridad es una prioridad para Amanda OS CLI Tools. Si descubres una vulnerabilidad de seguridad, te pedimos que la reportes de manera responsable.

### Proceso de Reporte

**NO abras un issue público** para reportar vulnerabilidades de seguridad.

En su lugar, por favor:

1. **Envía un email** a: corinoah1013@github.com
2. **Incluye**:
   - Descripción detallada de la vulnerabilidad
   - Pasos para reproducir (PoC si es posible)
   - Impacto potencial
   - Sugerencias de mitigación (opcional)

### Qué Esperar

- **Confirmación** de recepción dentro de 48 horas
- **Evaluación** inicial dentro de 1 semana
- **Actualizaciones** periódicas sobre el progreso
- **Crédito** en el changelog/advisory (si lo deseas)

### Alcance

Vulnerabilidades relevantes incluyen pero no se limitan a:

- **amanda-core**:
  - Debilidades en el hash chain de `.amaudit`
  - Problemas de serialización/deserialización
  - Manipulación de configuración (`.amconf`)

- **amanda-watch**:
  - Escalación de privilegios
  - Fugas de información sensible en snapshots
  - Condiciones de carrera en monitoreo

- **amanda-logs** (futuro):
  - Parsing inseguro de logs
  - Bypass de verificación de integridad

- **amanda-mail** (futuro):
  - Problemas de TLS/SSL
  - Vulnerabilidades de parsing IMAP/MIME

### Prácticas de Seguridad

El proyecto sigue estas prácticas:

- Dependencias actualizadas regularmente
- Auditoría de dependencias con `cargo audit`
- Uso de funciones seguras de Rust (sin `unsafe` innecesario)
- Validación de entrada estricta
- Tests de seguridad automatizados

## Historial de Advisories

| Fecha | Versión | Descripción | CVE |
|-------|---------|-------------|-----|
| - | - | Sin advisories reportados | - |

---

Agradecemos tu ayuda para mantener Amanda OS CLI Tools seguro.
