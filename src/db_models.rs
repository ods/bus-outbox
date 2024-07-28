use sqlx::types::JsonValue;

#[derive(Debug)]
pub(crate) struct OutboxMessage {
    pub id: i64,
    pub topic: String,
    pub payload: Option<Vec<u8>>,
    pub key: Option<Vec<u8>>,
    // TODO: Use `Option<Json<HashMap<String, Option<String>>>>`
    pub headers: Option<JsonValue>,
}
