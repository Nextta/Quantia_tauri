use crate::api::resultados::get_resultados_by_id_backtest;
use crate::backtest::resultados::Resultados;

/// Optiene los datos de los resultados de un backtest.
///
/// # Argments:
/// id: Identificador del backtest al que vamos a optener los datos de los resultados.
///
/// # Return
/// Delvuelve los resultados del backtest.
#[tauri::command]
pub async fn get_results_by_backtest(id: i32) -> Resultados {
    let resultados: Resultados = get_resultados_by_id_backtest(id)
        .await
        .unwrap_or(Resultados::new(id).await);

    resultados
}
