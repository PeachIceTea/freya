use serde::Serialize;
use time::OffsetDateTime;

#[derive(Serialize)]
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
pub struct SessionDBEntry {
    pub id: String,
    pub user_id: i64,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub last_accessed: OffsetDateTime,
}

#[derive(Serialize)]
pub struct Book {
    pub id: i64,

    pub title: String,
    pub author: String,
    pub cover: Option<Vec<u8>>,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}

#[derive(Serialize)]
pub struct File {
    pub id: i64,
    pub book_id: i64,

    pub path: String,

    pub name: String,
    pub position: i32,
    pub duration: f64,

    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub modified: OffsetDateTime,
}
