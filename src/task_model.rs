use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub total_duration_seconds: u64,
    pub running: bool,
    pub last_run: Option<SystemTime>,
}
