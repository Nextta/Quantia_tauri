# API - Quantia Tauri (Backend)

## Comandos disponibles

Todos los comandos se invocan desde el frontend mediante `invoke('comando', { parametros })`.

---

## 1. Backtests (`backtests.rs`)

### `get_alls_backtests`

Obtiene todos los backtests guardados.

**Parámetros:** Ninguno

**Respuesta:** `Vec<Backtest>`

```json
[
  {
    "id": 1,
    "titulo": "Cruce de medias EURUSD",
    "balance": 10000.0,
    "tipo": "CDF",
    "trades": [...],
    "datos": { ... },
    "estrategia": { ... }
  }
]
```

**Ejemplo:**
```js
invoke('get_alls_backtests')
```

---

### `get_backtest`

Obtiene un backtest por su ID.

| Parámetro | Tipo   | Requerido | Descripción     |
|-----------|--------|-----------|-----------------|
| `id`      | `i32`  | Sí        | ID del backtest |

**Respuesta:** `Backtest`

**Ejemplo:**
```js
invoke('get_backtest', { id: 35 })
```

---

## 2. Datos OHLC / Ticks (`data.rs`)

### `save_data_dukas`

Guarda datos OHLC descargados (velas con open, high, low, close, volume) en el formato y ubicación especificados. No apto para ticks.

| Parámetro     | Tipo              | Requerido | Descripción                                    |
|---------------|-------------------|-----------|------------------------------------------------|
| `data`        | `Vec<DataDukas>`  | Sí        | Array de velas                                  |
| `name`        | `String`          | Sí        | Nombre del archivo/símbolo                     |
| `timeframe`   | `Timeframe`       | Sí        | Timeframe de los datos (ej: "M1", "H1")        |
| `from_date`   | `String`          | Sí        | Fecha inicio ("YYYY-MM-DD HH:MM:SS")           |
| `to_date`     | `String`          | Sí        | Fecha fin ("YYYY-MM-DD HH:MM:SS")              |
| `broker_data` | `DataOrigen`      | Sí        | Origen ("DukasCopy", "MT5", "Import")          |
| `ruta`        | `Option<String>`  | No        | Carpeta destino (default: "download")          |
| `format`      | `Option<DataFormatSymbol>` | No | Formato ("Parquet", "Csv", "Json")             |
| `actualized`  | `Option<bool>`    | No        | Si está actualizado (default: false)           |

**Estructura `DataDukas`:**
```typescript
interface DataDukas {
  timestamp: number;  // Unix timestamp en milisegundos
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}
```

**Respuesta:** `String` — Mensaje de confirmación.

**Ejemplo:**
```js
invoke('save_data_dukas', {
  data: [
    { timestamp: 1547416809672, open: 1.2525, high: 1.2336, low: 1.2222, close: 1.2563, volume: 12524.0 }
  ],
  name: "EURUSD",
  timeframe: "M1",
  from_date: "2019-01-01 00:00:00",
  to_date: "2019-12-31 23:59:59",
  broker_data: "DukasCopy",
  format: "Csv",
  actualized: true
})
```

---

### `save_data_dukas_ticks`

Guarda datos de ticks descargados (askPrice, bidPrice, askVolume, bidVolume). Solo para timeframe Ticks.

| Parámetro     | Tipo                  | Requerido | Descripción                                  |
|---------------|-----------------------|-----------|----------------------------------------------|
| `data`        | `Vec<DataDukasTicks>` | Sí        | Array de ticks                               |
| `name`        | `String`              | Sí        | Nombre del archivo/símbolo                   |
| `timeframe`   | `Timeframe`           | Sí        | Debe ser "Ticks"                             |
| `from_date`   | `String`              | Sí        | Fecha inicio                                 |
| `to_date`     | `String`              | Sí        | Fecha fin                                    |
| `broker_data` | `DataOrigen`          | Sí        | Origen de datos                              |
| `ruta`        | `Option<String>`      | No        | Carpeta destino (default: "download")        |
| `format`      | `Option<DataFormatSymbol>` | No | Formato ("Parquet", "Csv", "Json")           |
| `actualized`  | `Option<bool>`        | No        | Si está actualizado (default: false)         |

**Estructura `DataDukasTicks`:**
```typescript
interface DataDukasTicks {
  timestamp: number;   // Unix timestamp en milisegundos
  askPrice: number;
  bidPrice: number;
  askVolume: number;
  bidVolume: number;
}
```

**Respuesta:** `String` — Mensaje de confirmación.

**Ejemplo:**
```js
invoke('save_data_dukas_ticks', {
  data: [
    { timestamp: 1547416809672, askPrice: 1.2221, bidPrice: 1.2220, askVolume: 1.5, bidVolume: 2.3 }
  ],
  name: "EURUSD_TICKS",
  timeframe: "Ticks",
  from_date: "2019-01-01 00:00:00",
  to_date: "2019-12-31 23:59:59",
  broker_data: "DukasCopy",
  format: "Csv"
})
```

---

### `get_data_for_tv`

Obtiene datos OHLC de un backtest para gráficos TradingView, con filtro por rango de fechas y velas extra alrededor.

| Parámetro    | Tipo             | Requerido | Descripción                                           |
|-------------|------------------|-----------|-------------------------------------------------------|
| `id_backtest`| `i32`           | Sí        | ID del backtest                                       |
| `from_date`  | `&str`          | Sí        | Fecha inicio ("YYYY-MM-DD HH:MM:SS")                  |
| `to_date`    | `&str`          | Sí        | Fecha fin ("YYYY-MM-DD HH:MM:SS")                     |
| `prev_bars`  | `Option<u32>`   | No        | Velas extra antes/después del rango (default: 2000)   |

**Respuesta:** `Vec<DataTv>`
```typescript
interface DataTv {
  time: number;    // Unix timestamp en segundos
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}
```

**Ejemplo:**
```js
invoke('get_data_for_tv', {
  id_backtest: 35,
  from_date: "2024-08-19 04:00:00",
  to_date: "2024-08-19 11:00:00",
  prev_bars: 40
})
```

---

## 3. Indicadores (`indicadores.rs`)

### `get_indicator_for_tv`

Obtiene una serie temporal de un indicador específico calculado sobre los datos de un backtest, para usar en gráficos TradingView.

| Parámetro       | Tipo             | Requerido | Descripción                                           |
|----------------|------------------|-----------|-------------------------------------------------------|
| `id_backtest`  | `i32`            | Sí        | ID del backtest                                       |
| `id_estrategia`| `i32`            | Sí        | ID de la estrategia con los indicadores a calcular    |
| `column`       | `&str`           | Sí        | Nombre exacto de la columna del indicador (ej: "ema_50") |
| `from_date`    | `&str`           | Sí        | Fecha inicio ("YYYY-MM-DD HH:MM:SS")                  |
| `to_date`      | `&str`           | Sí        | Fecha fin ("YYYY-MM-DD HH:MM:SS")                     |
| `prev_bars`    | `Option<u32>`    | No        | Velas extra antes/después del rango (default: 2000)   |

Nota: La estrategia se carga desde la BD usando `id_estrategia`. Los indicadores se calculan en tiempo real sobre los datos del backtest.

**Respuesta:** `Vec<DataIndicator>`
```typescript
interface DataIndicator {
  time: number;   // Unix timestamp en segundos
  value: number;  // Valor del indicador en esa vela
}
```

**Ejemplo:**
```js
invoke('get_indicator_for_tv', {
  id_backtest: 35,
  id_estrategia: 1,
  column: "ema_50",
  from_date: "2024-08-19 04:00:00",
  to_date: "2024-08-19 11:00:00",
  prev_bars: 40
})
```

**Columnas típicas disponibles** (dependen de la estrategia):
- Medias móviles: `sma_20`, `ema_50`, `sma_200`, etc.
- Bandas de Bollinger: `bb_upper`, `bb_middle`, `bb_lower`
- RSI: `rsi_14`
- MACD: `macd`, `macd_signal`, `macd_histogram`
- Cualquier indicador definido en la estrategia

---

## 4. Resultados (`resultados.rs`)

### `get_results_by_backtest`

Obtiene las métricas de rendimiento de un backtest.

| Parámetro | Tipo   | Requerido | Descripción     |
|-----------|--------|-----------|-----------------|
| `id`      | `i32`  | Sí        | ID del backtest |

**Respuesta:** `Resultados` — Objeto con ~68 métricas financieras.

**Métricas principales incluidas:**
| Campo                | Tipo    | Descripción                            |
|----------------------|---------|----------------------------------------|
| `retorno`            | `f64`   | Retorno absoluto en divisa             |
| `return_percent`     | `f64`   | Retorno porcentual                     |
| `cagr`               | `f64`   | Compound Annual Growth Rate            |
| `sharpe_ratio`       | `f64`   | Ratio de Sharpe                        |
| `sortino_ratio`      | `f64`   | Ratio de Sortino                       |
| `max_drawdown`       | `f64`   | Drawdown máximo porcentual             |
| `profit_factor`      | `f64`   | Profit factor                          |
| `n_trades`           | `u64`   | Número total de trades                 |
| `wins_percentage`    | `f64`   | Porcentaje de operaciones ganadoras    |
| `avg_trade_return`   | `f64`   | Retorno promedio por trade             |
| `kelly_criterion`    | `f64`   | Criterio de Kelly                      |
| `payoff_ratio`       | `f64`   | Payoff ratio                           |

**Ejemplo:**
```js
invoke('get_results_by_backtest', { id: 35 })
```

---

## 5. Trades (`trades.rs`)

### `get_tardes`

Obtiene todas las operaciones (trades) de un backtest.

| Parámetro    | Tipo   | Requerido | Descripción     |
|-------------|--------|-----------|-----------------|
| `id_backtest`| `i32`  | Sí        | ID del backtest |

**Respuesta:** `Vec<Trade>`

**Ejemplo:**
```js
invoke('get_tardes', { id_backtest: 35 })
```

---

### `get_tardes_page`

Obtiene trades paginados de un backtest.

| Parámetro    | Tipo   | Requerido | Descripción                     |
|-------------|--------|-----------|---------------------------------|
| `id_backtest`| `i32`  | Sí        | ID del backtest                 |
| `limite`     | `i32`  | Sí        | Cantidad de trades por página   |
| `pagina`     | `i32`  | Sí        | Número de página (0-based)      |

**Respuesta:** `Vec<Trade>`

**Ejemplo:**
```js
invoke('get_tardes_page', { id_backtest: 35, limite: 20, pagina: 0 })
```

---

### `get_trade`

Obtiene un trade específico por su ID.

| Parámetro | Tipo   | Requerido | Descripción |
|-----------|--------|-----------|-------------|
| `id`      | `i32`  | Sí        | ID del trade |

**Respuesta:** `Trade`

**Ejemplo:**
```js
invoke('get_trade', { id: 150 })
```

---

## 6. Usuarios (`users.rs`)

Gestión de usuarios de la aplicación. La autenticación se gestiona en el
frontend con Clerk; el backend solo almacena el `id_clerk` y los datos de
perfil. Todos los comandos de esta sección devuelven `Result` y propagan
errores (captúralos con `try/catch` en el frontend).

> **Seguridad:** la `clave_activacion` es un dato sensible; no la registres en
> logs ni la muestres en la consola del navegador.

### `table_users`

Crea la tabla de usuarios en la base de datos (idempotente).

**Parámetros:** Ninguno

**Respuesta:** `String` — Mensaje de confirmación.

**Ejemplo:**
```js
invoke('table_users')
```

---

### `insert_user`

Inserta un nuevo usuario. El `id_clerk` actúa como clave primaria: si ya
existe un usuario con ese id, el comando devuelve error.

| Parámetro | Tipo   | Requerido | Descripción                  |
|-----------|--------|-----------|------------------------------|
| `user`    | `User` | Sí        | Datos del usuario a insertar |

**Respuesta:** `void` (`null`) — Ok si la inserción es correcta.

**Ejemplo:**
```js
invoke('insert_user', {
  user: {
    id_clerk: "user_2abc...",
    nombre: "Nombre",
    apellidos: "Apellidos",
    username: "usuario1",
    descripcion: null,
    clave_activacion: null,
    usuario_activo: false
  }
})
```

---

### `get_users`

Obtiene todos los usuarios.

**Parámetros:** Ninguno

**Respuesta:** `Vec<User>`

**Ejemplo:**
```js
invoke('get_users')
```

---

### `get_user_by_id_clerk`

Obtiene un usuario por su identificador de Clerk.

| Parámetro  | Tipo     | Requerido | Descripción             |
|------------|----------|-----------|-------------------------|
| `id_clerk` | `String` | Sí        | ID del usuario en Clerk |

**Respuesta:** `User` — Error si no existe.

**Ejemplo:**
```js
invoke('get_user_by_id_clerk', { id_clerk: "user_2abc..." })
```

---

### `get_user_by_username`

Obtiene un usuario por su username.

| Parámetro  | Tipo     | Requerido | Descripción |
|------------|----------|-----------|-------------|
| `username` | `String` | Sí        | Username    |

**Respuesta:** `User` — Error si no existe.

**Ejemplo:**
```js
invoke('get_user_by_username', { username: "usuario1" })
```

---

### `update_user`

Actualiza el perfil de un usuario (nombre, apellidos, username y descripción).
No modifica `clave_activacion` ni `usuario_activo`.

| Parámetro | Tipo   | Requerido | Descripción                                 |
|-----------|--------|-----------|---------------------------------------------|
| `user`    | `User` | Sí        | Datos nuevos, con el `id_clerk` del usuario |

**Respuesta:** `void` (`null`)

**Ejemplo:**
```js
invoke('update_user', {
  user: {
    id_clerk: "user_2abc...",
    nombre: "NombreNuevo",
    apellidos: "Apellidos",
    username: "usuario1",
    descripcion: "Trader de divisas",
    clave_activacion: null,
    usuario_activo: false
  }
})
```

---

### `update_user_clave`

Asigna o actualiza la clave de activación de un usuario.

| Parámetro  | Tipo     | Requerido | Descripción               |
|------------|----------|-----------|---------------------------|
| `id_clerk` | `String` | Sí        | ID del usuario en Clerk   |
| `clave`    | `String` | Sí        | Nueva clave de activación |

**Respuesta:** `void` (`null`)

**Ejemplo:**
```js
invoke('update_user_clave', { id_clerk: "user_2abc...", clave: "XXXX-XXXX" })
```

---

### `update_user_activo`

Activa o desactiva el acceso de un usuario al programa.

| Parámetro  | Tipo      | Requerido | Descripción                     |
|------------|-----------|-----------|---------------------------------|
| `id_clerk` | `String`  | Sí        | ID del usuario en Clerk         |
| `activo`   | `boolean` | Sí        | `true` = activado, `false` = no |

**Respuesta:** `void` (`null`)

**Ejemplo:**
```js
invoke('update_user_activo', { id_clerk: "user_2abc...", activo: true })
```

---

### `delete_user`

Elimina un usuario por su identificador de Clerk.

| Parámetro  | Tipo     | Requerido | Descripción             |
|------------|----------|-----------|-------------------------|
| `id_clerk` | `String` | Sí        | ID del usuario en Clerk |

**Respuesta:** `void` (`null`)

**Ejemplo:**
```js
invoke('delete_user', { id_clerk: "user_2abc..." })
```

---

## 7. Tipos de datos compartidos

### `Backtest`
```typescript
interface Backtest {
  id: number;
  titulo: string;
  balance: number;
  tipo: "Forex" | "Futuros" | "CDF" | "Acciones" | "ETF" | "Opciones";
  gestion_strategy: string;
  parametros_gestion: {
    multiplicador: number;
    lotaje_fijo: number;
  };
  trades: Trade[];
  datos: DataSymbol;
  estrategia: Strategy;  // incluye indicadores y condiciones
}
```

### `Trade`
```typescript
interface Trade {
  id: number;
  id_backtest: number;
  id_symbol: number;
  symbol: SymbolInfoCFD;   // Información del símbolo negociado
  tipo: "Buy" | "Sell";    // Dirección de la operación
  lotaje: number;
  multiplicador: number;
  t0: string;               // Fecha/hora de entrada
  precio_entrada: number;
  tp: number;               // Take profit
  sl: number;               // Stop loss
  t1: string;               // Fecha/hora de cierre
  precio_cierre: number;
  precio_maximo: number;    // Precio máximo alcanzado
  precio_minimo: number;    // Precio mínimo alcanzado
  duracion_segundos: string;
  duracion_minutos: string;
  duracion_horas: string;
  duracion_dias: string;
  label: number;            // 1 = ganada, 0 = perdida
  pl: number;               // P&L con comisiones
  plsc: number;             // P&L sin comisiones
  pips_pl: number;          // P&L en pips
}
```

### `Resultados`
Objeto con 68 métricas financieras. Las principales están documentadas en la sección `get_results_by_backtest`.

### `DataTv`
```typescript
interface DataTv {
  time: number;    // Unix timestamp en segundos
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}
```

### `DataIndicator`
```typescript
interface DataIndicator {
  time: number;   // Unix timestamp en segundos
  value: number;  // Valor del indicador
}
```

### `DataSymbol`
```typescript
interface DataSymbol {
  id: number;
  name: string;
  timeframe: string | null;  // "Ticks" | "M1" | "M5" | "M10" | "M15" | "M30" | "H1" | "H4" | "D1" | "W1" | "MM1"
  ruta: string;
  formato: string | null;    // "Parquet" | "Csv" | "Json"
  fecha_inicio: string;
  fecha_fin: string;
  actualizado: boolean;
  n_data: number;
  origen: string | null;     // "DukasCopy" | "MT5" | "Import"
}
```

### `User`
```typescript
interface User {
  id_clerk: string;            // ID del usuario en Clerk (clave primaria)
  nombre: string;
  apellidos: string;
  username: string;
  descripcion: string | null;
  clave_activacion: string | null;
  usuario_activo: boolean;
}
```

---

## 8. Enumeraciones

### `Timeframe`
```typescript
type Timeframe = "Ticks" | "M1" | "M5" | "M10" | "M15" | "M30" | "H1" | "H4" | "D1" | "W1" | "MM1";
```

### `DataOrigen`
```typescript
type DataOrigen = "DukasCopy" | "MT5" | "Import";
```

### `DataFormatSymbol`
```typescript
type DataFormatSymbol = "Parquet" | "Csv" | "Json";
```

### `Activo` (tipo de activo)
```typescript
type Activo = "Forex" | "Futuros" | "CDF" | "Acciones" | "ETF" | "Opciones";
```

### `EntryDirection`
```typescript
type EntryDirection = "Buy" | "Sell";
```

---

## Notas importantes

1. **Fechas**: Todos los parámetros de fecha se pasan como string en formato `"YYYY-MM-DD HH:MM:SS"`.
2. **Timestamps**: Los valores `time` en las respuestas están en **segundos** (Unix epoch). Los timestamps en los datos de entrada (`DataDukas.timestamp`) deben estar en **milisegundos**.
3. **Velas extra**: `prev_bars` controla cuántas velas adicionales se incluyen antes del `from_date` y después del `to_date`, útil para cálculos de indicadores que necesitan contexto histórico.
4. **Columnas de indicadores**: El parámetro `column` en `get_indicator_for_tv` debe coincidir exactamente con el nombre de la columna generada por la estrategia (ej: `"sma_20"`, `"ema_50"`, `"rsi_14"`).
5. **Estrategia**: En `get_indicator_for_tv` la estrategia se carga por separado (`id_estrategia`), no desde el backtest.
6. **Paginación**: `get_tardes_page` usa paginación 0-based (página 0 = primeras `limite` filas).
