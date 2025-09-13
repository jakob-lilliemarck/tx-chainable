use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub payload: Value,
}
