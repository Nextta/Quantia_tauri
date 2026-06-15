pub mod api;
pub mod backtest;
pub mod comandos;
pub mod data_lab;
pub mod enums;
pub mod estrategias;
pub mod indicators;
pub mod strategy;
pub mod structs;
pub mod traits;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            comandos::backtests::get_alls_backtests,
            comandos::resultados::get_results_by_backtest,
            comandos::trades::get_trade,
            comandos::trades::get_tardes,
            comandos::trades::get_tardes_page
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
