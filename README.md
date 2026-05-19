# QuantiaPro Builder

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)

**Advanced algorithmic trading strategy builder and backtester**

## 🎯 Descripción

QuantiaPro Builder es una plataforma avanzada para construir, probar y ejecutar estrategias de trading algorítmico. Combina un motor de backtesting de alto rendimiento con una interfaz desktop moderna construida con Tauri y Astro.

<!--## 📁 Estructura del Proyecto

Este proyecto utiliza un workspace monorepo de Rust:

```
quantiaPro_Builder/
├── crates/
│   ├── quantia_core/          # Motor principal: backtester, indicadores, DSL, ML
│   └── quantia_persistence/   # Capa de persistencia SQLite
├── apps/
│   └── quantia_desktop/       # Aplicación desktop con Tauri
├── Cargo.toml                 # Workspace raíz
└── README.md
```-->

## 🚀 Inicio Rápido

### Requisitos Previos

- Rust 1.75 o superior
- Node.js 18+ (para la aplicación desktop)
- SQLite 3

### Instalación

1. Clonar el repositorio:
```bash
git clone https://github.com/Nextta/QuantiaProBuilder.git
cd QuantiaProBuilder
```

2. Compilar el workspace:
```bash
cargo build
```

3. Ejecutar tests:
```bash
cargo test
```

## 🧩 Crates

### quantia_core
Motor principal que incluye:
- Backtester de alto rendimiento
- Biblioteca de indicadores técnicos
- DSL para definir estrategias
- Modelos de machine learning

### quantia_persistence
Capa de persistencia con:
- Repositorios SQLite
- Sistema de migraciones
- Gestión de estrategias y backtests

### quantia_desktop
Aplicación desktop con:
- Frontend: Astro + TypeScript
- Backend: Tauri + Rust
- Interfaz moderna y reactiva

## 📝 Licencia

Este proyecto está bajo la licencia MIT. Ver el archivo LICENSE para más detalles.

## 👥 Contribuciones

En estos momentos el proyecto esta en privado en fase de desarrollo.
<!--Las contribuciones son bienvenidas. Por favor, abre un issue o un pull request.-->

## 🔗 Enlaces

- Repositorio: https://github.com/Nextta/Quantia_tauri.git
- Website: https://www.quantiapro.com

---

**Desarrollado por QuantiaPro Team**
