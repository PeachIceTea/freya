use serde::Serialize;
use time::OffsetDateTime;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,

    pub name: String,

    #[serde(skip)]
    pub password: String,

    pub admin: bool,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDBEntry {
    pub id: String,
    pub user_id: i64,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub last_accessed: OffsetDateTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: i64,

    pub title: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: i64,
    pub book_id: i64,

    #[serde(skip)]
    pub path: String,

    pub name: String,
    pub position: i64,
    pub duration: f64,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntry {
    pub id: i64,

    #[serde(skip)]
    pub user_id: i64,
    #[serde(skip)]
    pub book_id: i64,

    pub file_id: i64,
    pub progress: f64,

    pub list: String,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}
