pub use bson::Uuid;
use chrono::{DateTime, Local};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn to_be_bytes(hash: &str) -> [u8; 16] {
    hash.as_bytes().try_into().unwrap()
}

fn calculate_hash<T: Hash>(t: &[T]) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish().to_string().chars().take(16).collect()
}

pub fn generate(seed: [&String; 6]) -> Uuid {
    let hash = calculate_hash(&seed);
    let hash_bytes = to_be_bytes(&hash);
    Uuid::from_bytes(hash_bytes)
}

pub fn from_str(uuid: String) -> Uuid {
    Uuid::parse_str(uuid).unwrap()
}

pub fn generate_ts_id(date: DateTime<Local>) -> usize {
    (date.timestamp_millis() / 1000) as usize
}
