use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterStats {
    appearances: u32,
    successes: u32,
    failures: u32,
    total_response_time: f64,
    last_appearance: DateTime<Utc>,
}

impl CharacterStats {
    pub fn success_rate(&self) -> f64 {
        if self.appearances == 0 {
            return 0.0;
        }
        self.successes as f64 / self.appearances as f64
    }

    pub fn avg_response_time(&self) -> f64 {
        if self.appearances == 0 {
            return 0.0;
        }
        self.total_response_time / self.appearances as f64
    }

    pub fn calculate_weight(&self, now: DateTime<Utc>) -> f64 {
        let recency_factor = (now - self.last_appearance).num_hours() as f64 / 24.0;
        let error_rate = 1.0 - self.success_rate();
        let response_factor = (self.avg_response_time() / 1000.0).min(1.0);
        
        1.0 + (error_rate * 2.0) + (response_factor * 0.5) + (recency_factor * 0.3)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserHistory {
    character_stats: HashMap<String, CharacterStats>,
    last_session: DateTime<Utc>,
    total_practice_time: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum PracticeMode {
    Main,
    Dakuten,
    Combination,
    All,
}