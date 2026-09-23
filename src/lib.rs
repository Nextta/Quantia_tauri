pub mod api;
pub mod backtest;
pub mod comandos;
pub mod data_lab;
pub mod enums;
pub mod indicators;
pub mod strategy;
pub mod structs;
pub mod traits;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            comandos::backtests::get_alls_backtests,
            comandos::resultados::get_results_by_backtest,
            comandos::trades::get_trade,
            comandos::trades::get_trades,
            comandos::trades::get_trades_page,
            comandos::backtests::get_backtest,
            comandos::data::save_data_dukas,
            comandos::data::save_data_dukas_ticks,
            comandos::data::get_data_for_tv,
            comandos::indicadores::get_indicator_for_tv,
            comandos::users::table_users,
            comandos::users::insert_user,
            comandos::users::get_users,
            comandos::users::get_user_by_id_clerk,
            comandos::users::get_user_by_username,
            comandos::users::update_user,
            comandos::users::update_user_clave,
            comandos::users::update_user_activo,
            comandos::users::delete_user,
            comandos::activation_keys::table_activation_keys,
            comandos::activation_keys::insert_key,
            comandos::activation_keys::get_keys,
            comandos::activation_keys::get_available_keys,
            comandos::activation_keys::get_key_by_id,
            comandos::activation_keys::delete_key,
            comandos::activation_keys::generate_keys,
            comandos::activation_keys::activar_usuario,
            comandos::activation_keys::release_key,
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
