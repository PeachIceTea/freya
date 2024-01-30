// json macro that combines serde_json::json! and axum::Json
#[macro_export]
macro_rules! axum_json {
    ($($json:tt)*) => {
        axum::Json(serde_json::json!($($json)*))
    };
}
