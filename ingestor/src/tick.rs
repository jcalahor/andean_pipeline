use chrono::{DateTime, NaiveDateTime};
use serde::{Serialize, Deserialize};
use chrono::{Utc};

#[derive(Serialize, Deserialize, Debug)]
pub struct Tick {
    pub Symbol: String,
    pub Price: f32,
    pub TimeStamp: i64
}