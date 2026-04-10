use rust_decimal::Decimal;

pub fn truncate_decimal(valor: Decimal, decimales: u32) -> Decimal {
    let potencia = Decimal::from(10_i64.pow(decimales));
    (valor * potencia).trunc() / potencia
}
