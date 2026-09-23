# AGENTS.md — Quantia_tauri

## Proyecto
Backend Rust de **QuantiaPro Builder**: aplicación desktop (Tauri 2) para construir,
hacer backtesting y ejecutar estrategias de trading algorítmico.
El frontend (Astro + TypeScript) vive en otro repositorio y se referencia en
`Quantia_astro/` (hermano de este proyecto).
Stack: Rust (edition 2021), Tauri 2, Polars (dataframes), libsql (SQLite), Tokio, Serde, Reqwest.

Estructura clave de `src/`:
- `comandos/` — comandos Tauri expuestos al frontend (documentados en `src/comandos/API.md`)
- `api/` — acceso a datos (backtests, trades, estrategias, símbolos)
- `backtest/` — motor de backtesting
- `indicators/` — biblioteca de indicadores técnicos
- `data_lab/` — importación/exportación/gestión de datos (Dukascopy, etc.)
- `strategy/`, `structs/`, `enums/`, `traits/`, `utils/` — dominio y utilidades

## Comandos
- Ejecutar (dev): `cargo tauri dev`
- Compilar: `cargo build`
- Tests: `cargo test`
- Lint/formato: `cargo fmt` / `cargo clippy`

## Estilo y convenciones
- Rust edition 2021 (rust-version 1.77.2), snake_case estándar.
- Código, comentarios y documentación en español.
- Cada comando Tauri nuevo debe documentarse en `src/comandos/API.md`.
- Tests inline con `#[cfg(test)]` en el mismo archivo que el código.

## Reglas
- Lee `docs/constitution.md` antes de tocar código
- Lee `specs/` antes de realizar una tarea en el proyecto y mantenlo actualizado.
- Haz las preguntas necesarias para entender la tarea antes de comenzarla.
- Lee `src/comandos/API.md` antes de tocar comandos Tauri y mantenlo actualizado.
- No tocar sin preguntar: `Quantia.db`, `.env`, `target/`, `gen/`, `capabilities/`,
  `tauri.conf.json`.
- No añadir dependencias (crates) ni features de Polars sin consultar.
- No renombrar ni eliminar comandos Tauri existentes (rompe el frontend).
- Documenta todo el código que crees en español.

## Al terminar cualquier tarea
- Ejecutar `cargo test` y verificar que compila (`cargo check`).
