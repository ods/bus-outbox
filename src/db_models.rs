use std::collections::HashMap;

use derive_more::Deref;
use serde::Deserialize;
use sqlx::{types::Json, FromRow};

#[derive(Debug, Deserialize, Deref)]
pub struct Headers(HashMap<String, Option<String>>);

#[derive(Debug, FromRow)]
pub struct OutboxMessage {
    pub id: i64,
    pub topic: String,
    pub payload: Option<Vec<u8>>,
    pub key: Option<Vec<u8>>,
    pub headers: Option<Json<Headers>>,
}
