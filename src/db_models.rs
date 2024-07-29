use std::collections::HashMap;

use serde::Deserialize;
use sqlx::{types::Json, FromRow};

#[derive(Debug, Deserialize)]
pub struct Headers(HashMap<String, Option<String>>);

#[derive(Debug, FromRow)]
pub struct OutboxMessage {
    pub id: i64,
    pub topic: String,
    pub payload: Option<Vec<u8>>,
    pub key: Option<Vec<u8>>,
    pub headers: Option<Json<Headers>>,
}
