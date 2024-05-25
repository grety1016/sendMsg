
pub use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
pub use rust_decimal::Decimal;
pub use tiberius::Uuid;

pub fn init() {
    let subscriber =
        tracing_subscriber::FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub fn conn_str() -> String {
    let host = "localhost";
    let database = "ZSKAIS20240101213214";
    let user = "sa";
    let pwd = "kephi";
    format!(
        r#"Server={};Database={};Uid={};Pwd="{}";TrustServerCertificate=true;"#,
        host, database, user, pwd
    )
}
