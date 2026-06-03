use crate::api::resultados::get_resultados_by_id_backtest;
use crate::backtest::resultados::Resultados;

#[tauri::command]
pub async fn get_results_by_backtest(id: i32) -> Resultados {
    let resultados: Resultados = get_resultados_by_id_backtest(id)
        .await
        .unwrap_or(Resultados::new(id).await);

    resultados
}
