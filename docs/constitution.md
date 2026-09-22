# Constitución — QuantiaPro Builder (Quantia_tauri)

**Versión:** 1.0.0 · **Fecha de ratificación:** 2026-09-22 · **Estado:** Activa

Documento de gobernanza del proyecto. Define los principios y límites con los
que se escribe, revisa y mantiene el código. Toda tarea, especificación
(`spec.md`, `plan.md`, `tasks.md`) y revisión debe ser coherente con esta
constitución. Una tarea solo puede contradecir un artículo si documenta la
excepción y su justificación en su `spec.md`, con aprobación del propietario.

---

## Artículo 1 — Naturaleza del proyecto

- Quantia_tauri es el **backend Rust** de QuantiaPro Builder: aplicación
  desktop (Tauri 2) para construir, hacer backtesting y ejecutar estrategias
  de trading algorítmico.
- El frontend (Astro + TypeScript) vive en otro repositorio (`Quantia_astro/`).
  Este repositorio **no contiene UI**.
- La única vía de comunicación frontend ↔ backend son los **comandos Tauri**
  (`invoke`), registrados en `src/lib.rs`.
- Stack: Rust (edition 2021), Tauri 2, Polars, libsql (SQLite), Tokio, Serde,
  Reqwest.

## Artículo 2 — Idioma y convenciones

- Documentación, comentarios, doc-comments, especificaciones y mensajes de
  commit: **español**.
- Código: Rust edition 2021, `snake_case` estándar. Se respeta la nomenclatura
  existente de cada módulo (tablas y campos de datos en español; comandos
  Tauri en inglés).
- Errores: las funciones que pueden fallar devuelven `Result<T, Error>` con el
  `Error` de `utils/configuracion.rs`. Evitar `unwrap()`/`expect()` fuera de
  tests y del arranque de Tauri.

## Artículo 3 — Arquitectura en capas

- `comandos/`: única puerta de entrada del frontend. Orquestan y delegan el
  acceso a datos en `api/` u otros módulos.
- `api/`: acceso a datos y persistencia. Un módulo por entidad
  (`backtests.rs`, `trades.rs`, `symbols.rs`, ...).
- `backtest/`, `indicators/`, `strategy/`: motor de dominio, desacoplados de
  la capa de comandos.
- Patrón estándar de un módulo de `api/`:
  - `table_<entidad>()`: crea su tabla con `CREATE TABLE IF NOT EXISTS`.
  - `insert_<entidad>()`, `get_<entidades>()`, `get_<entidad>_by_*()`,
    `update_*()` (si aplica) y `delete_<entidad>()`.
  - Conexión vía `get_db_config()` y `DB_LOCAL` (`Builder::new_local` o
    `Builder::new_remote_replica`), igual que el resto de módulos.
  - Doc-comment en español con secciones `# Parámetros`, `# Returns` y
    `# Errores`.

## Artículo 4 — Contrato API (comandos Tauri)

- Todo comando nuevo se documenta en `src/comandos/API.md` (parámetros,
  respuesta y ejemplo) **en la misma tarea** que lo implementa.
- Prohibido renombrar, eliminar o cambiar la firma de un comando ya
  registrado: rompe el frontend.
- Solo se registran en `lib.rs` los comandos que el frontend necesita.

## Artículo 5 — Datos y persistencia

- Base de datos SQLite vía libsql (`Quantia.db`). Esquemas creados con
  `CREATE TABLE IF NOT EXISTS` desde el módulo `api/` correspondiente.
- Toda tabla nueva o cambio de esquema requiere su `spec.md` aprobada antes
  de implementarse, y su CRUD completo en `api/`.
- Minimización de datos: solo se persiste lo estrictamente necesario. La
  autenticación la gestiona Clerk en el frontend; el backend solo guarda
  perfil/estado del usuario (p. ej. id de Clerk, datos de perfil, licencia),
  nunca credenciales ni secretos de Clerk.

## Artículo 6 — Seguridad

- `.env` nunca se toca, se muestra ni se versiona: contiene la configuración y
  credenciales de la base de datos.
- No exponer en logs rutas de ficheros, tokens, claves de activación ni datos
  personales de usuarios.

## Artículo 7 — Dependencias

- No añadir crates nuevos ni features de Polars sin consulta y aprobación
  previa del propietario.
- Preferir la librería estándar y las dependencias ya presentes.

## Artículo 8 — Flujo de trabajo (especificaciones)

Toda tarea no trivial se define antes de tocar código con tres documentos en
`specs/`:

- `spec.md` — **QUÉ**: requisitos, datos, casos de uso y criterios de
  aceptación.
- `plan.md` — **CÓMO**: enfoque técnico, diseño y decisiones.
- `tasks.md` — descomposición en pasos verificables y su orden.

Al terminar la tarea, las especificaciones y `API.md` se actualizan si el
resultado difiere de lo planificado.

## Artículo 9 — Verificación obligatoria

Antes de dar una tarea por cerrada:

1. `cargo check` compila sin errores ni warnings nuevos.
2. `cargo test` en verde. Los tests van inline con `#[cfg(test)]` en el mismo
   archivo que el código que prueban.
3. `cargo fmt` aplicado y `cargo clippy` sin warnings nuevos.
4. `API.md` y `specs/` actualizados si procede.

## Artículo 10 — Zonas protegidas

No modificar sin aprobación explícita del propietario:

- `Quantia.db` (base de datos real)
- `.env` y cualquier credencial
- `tauri.conf.json` y `capabilities/` (configuración y permisos de Tauri)
- `gen/` y `target/` (artefactos generados)
- Comandos Tauri existentes (nombre y firma)

## Artículo 11 — Ratificación y vigencia

- Esta constitución solo puede modificarse con aprobación del propietario del
  proyecto. Todo cambio incrementa la versión y actualiza la fecha.
- Si un artículo entra en conflicto con una necesidad concreta, se documenta
  la excepción en la `spec.md` de la tarea y se decide con el propietario
  antes de implementar.
- Se revisará cuando cambie la arquitectura, el stack o el modelo de datos.

---

## Checklist rápido (por tarea)

- [ ] Leí `AGENTS.md`, esta constitución y la spec activa.
- [ ] Existen `spec.md`, `plan.md` y `tasks.md` acordados.
- [ ] No toco zonas protegidas ni añado dependencias sin permiso.
- [ ] Nuevo comando → documentado en `API.md` y registrado en `lib.rs`.
- [ ] `cargo check` + `cargo test` + `cargo fmt`/`clippy` en verde.
- [ ] Documentación actualizada si el resultado difiere del plan.
