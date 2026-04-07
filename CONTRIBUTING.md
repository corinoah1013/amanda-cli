# Guía de Contribución

¡Gracias por tu interés en contribuir a Amanda OS CLI Tools! Este documento proporciona las pautas para contribuir al proyecto.

## Código de Conducta

Este proyecto y todos los participantes están gobernados por nuestro [Código de Conducta](CODE_OF_CONDUCT.md). Al participar, se espera que cumplas con este código.

## Cómo Contribuir

### Reportar Bugs

Si encuentras un bug, por favor abre un issue en GitHub con la siguiente información:

- **Título claro y descriptivo**
- **Descripción del bug**: Qué sucede vs. qué debería suceder
- **Pasos para reproducir**: Secuencia exacta de pasos
- **Comportamiento esperado**: Qué debería pasar
- **Comportamiento actual**: Qué sucede realmente
- **Entorno**:
  - OS y versión
  - Versión de Rust (`rustc --version`)
  - Versión de amanda-watch (`amanda-watch --version`)
- **Logs o output relevante**

### Solicitar Features

Para solicitar nuevas funcionalidades:

1. Verifica que no exista ya un issue similar
2. Abre un issue con el label `enhancement`
3. Describe claramente:
   - El problema que resuelve
   - La solución propuesta
   - Alternativas consideradas

### Pull Requests

1. **Fork** el repositorio
2. **Crea una rama** para tu feature (`git checkout -b feature/nombre-feature`)
3. **Commit** tus cambios (`git commit -m 'Add: nueva feature'`)
4. **Push** a tu fork (`git push origin feature/nombre-feature`)
5. Abre un **Pull Request**

#### Estándares de Código

- Sigue las convenciones de Rust (formato con `cargo fmt`)
- Asegúrate de que `cargo clippy` no reporte warnings
- Todos los tests deben pasar (`cargo test`)
- Documenta funciones públicas con docstrings
- Mantén el código simple y legible

#### Estructura de Commits

Usa prefijos descriptivos:
- `feat:` Nueva funcionalidad
- `fix:` Corrección de bug
- `docs:` Cambios en documentación
- `test:` Añadir o corregir tests
- `refactor:` Refactorización de código
- `chore:` Tareas de mantenimiento

Ejemplo:
```
feat: add memory threshold alerts for processes

- Add AlertConfig struct for configurable thresholds
- Implement AlertEngine for real-time monitoring
- Support both system and per-process alerts
```

## Desarrollo Local

### Setup

```bash
# Clonar
git clone https://github.com/corinoah1013/amanda-cli.git
cd amanda-cli

# Instalar Rust (si no lo tienes)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Compilar
cargo build --release

# Correr tests
cargo test --workspace
```

### Estructura del Proyecto

```
amanda-cli/
├── amanda-core/     # Biblioteca compartida (formatos .amaudit, .amrpt, .amconf)
├── amanda-watch/    # Herramienta de monitoreo
├── amanda-logs/     # Herramienta de logs (TODO)
└── amanda-mail/     # Cliente IMAP (TODO)
```

### Tests

```bash
# Todos los tests
cargo test --workspace

# Tests específicos
cargo test -p amanda-core
cargo test -p amanda-watch

# Con coverage (requiere cargo-tarpaulin)
cargo tarpaulin --workspace
```

### Linting y Formato

```bash
# Formatear código
cargo fmt --all

# Verificar formato
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings
```

## Roadmap

### En Progreso
- [ ] amanda-watch v0.2.0: Mejoras de UI y nuevos filtros

### Próximos
- [ ] amanda-logs: Análisis de logs con hash chain
- [ ] amanda-mail: Cliente IMAP para automatización

### Ideas
- [ ] Dashboard web para visualizar snapshots
- [ ] Plugin system para extensibilidad
- [ ] Soporte para Windows

## Reconocimientos

Los contribuyentes serán reconocidos en el archivo README.md del proyecto.

## Preguntas

Si tienes preguntas, puedes:
- Abrir un issue con el label `question`
- Contactar al mantenedor: corinoah1013@github.com

¡Gracias por contribuir!
