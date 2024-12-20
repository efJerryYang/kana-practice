use crate::kana::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistakeEntry {
    pub input: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEntry {
    pub input: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: f64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub appearances: u32,
    pub successes: u32,
    pub failures: u32,
    pub total_response_time: f64,
    pub last_appearance: DateTime<Utc>,
    pub exp_avg_response: f64,
    pub exp_avg_accuracy: f64,
    pub mistakes: Vec<MistakeEntry>,
    pub test_history: Vec<TestEntry>,
}

impl CharacterStats {
    const ALPHA: f64 = 0.2;

    pub fn new() -> Self {
        Self {
            appearances: 0,
            successes: 0,
            failures: 0,
            total_response_time: 0.0,
            exp_avg_response: 0.0,
            exp_avg_accuracy: 0.0,
            last_appearance: Utc::now(),
            mistakes: Vec::new(),
            test_history: Vec::new(),
        }
    }

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
        // Base weight for characters never seen
        if self.appearances == 0 {
            debug!(
                appearances = 0,
                weight = 3.0,
                "Base weight for unseen character"
            );
            return 3.0; // Maximum possible weight for new characters
        }

        // 1. Error rate component (0-1, higher is better for practice)
        let error_component = 1.0 - self.exp_avg_accuracy;

        // 2. Recency component (0-1, higher means needs practice)
        // Convert to seconds for finer granularity
        let seconds_since = (now - self.last_appearance).num_seconds() as f64;
        // Normalize over 1 hour (3600 seconds) with sigmoid function
        let recency_component = 1.0 / (1.0 + (-0.002 * (seconds_since - 1800.0)).exp());

        // 3. Response time component (0-1, higher means needs practice)
        // Using 1200ms as the median response time
        let response_component = 1.0 / (1.0 + (-0.005 * (self.exp_avg_response - 1200.0)).exp());

        // Calculate final weight as average of components plus base
        let components_avg = (error_component + recency_component + response_component) / 3.0;
        let weight = 1.0 + components_avg;

        debug!(
            error_component = error_component,
            recency_component = recency_component,
            response_component = response_component,
            seconds_since = seconds_since,
            exp_avg_accuracy = self.exp_avg_accuracy,
            exp_avg_response = self.exp_avg_response,
            components_avg = components_avg,
            final_weight = weight,
            appearances = self.appearances,
            "Weight calculation details"
        );

        weight
    }

    pub fn get_weight_components(&self, now: DateTime<Utc>) -> (f64, f64, f64) {
        if self.appearances == 0 {
            return (5.0, 0.0, 0.0);
        }

        let hours_since = (now - self.last_appearance).num_hours().min(24) as f64;
        let recency_factor = hours_since / 24.0;
        let error_rate = 1.0 - self.exp_avg_accuracy;
        let response_factor = (self.exp_avg_response / 5000.0).min(1.0);

        (
            error_rate * 3.0,
            response_factor * 0.5,
            recency_factor * 1.0,
        )
    }

    pub fn record_attempt(&mut self, input: &str, success: bool, response_time: f64) {
        self.appearances += 1;

        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
            self.mistakes.push(MistakeEntry {
                input: input.to_string(),
                timestamp: Utc::now(),
            });
        }

        self.test_history.push(TestEntry {
            input: input.to_string(),
            start_time: Utc::now() - chrono::Duration::milliseconds(response_time as i64),
            duration_ms: response_time,
            success,
        });

        if self.appearances == 1 {
            self.exp_avg_response = response_time;
            self.exp_avg_accuracy = if success { 1.0 } else { 0.0 };
        } else {
            self.exp_avg_response =
                Self::ALPHA * response_time + (1.0 - Self::ALPHA) * self.exp_avg_response;
            self.exp_avg_accuracy = Self::ALPHA * (if success { 1.0 } else { 0.0 })
                + (1.0 - Self::ALPHA) * self.exp_avg_accuracy;
        }

        self.total_response_time += response_time;
        self.last_appearance = Utc::now();
    }

    pub fn get_ema_accuracy(&self) -> f64 {
        self.exp_avg_accuracy
    }

    pub fn recalculate_ema(&mut self) {
        self.exp_avg_response = 0.0;
        self.exp_avg_accuracy = 0.0;

        for (i, entry) in self.test_history.iter().enumerate() {
            if i == 0 {
                self.exp_avg_response = entry.duration_ms;
                self.exp_avg_accuracy = if entry.success { 1.0 } else { 0.0 };
            } else {
                self.exp_avg_response =
                    Self::ALPHA * entry.duration_ms + (1.0 - Self::ALPHA) * self.exp_avg_response;
                self.exp_avg_accuracy = Self::ALPHA * (if entry.success { 1.0 } else { 0.0 })
                    + (1.0 - Self::ALPHA) * self.exp_avg_accuracy;
            }
        }
    }

    pub fn get_recent_avg_response_time(&self, n: usize) -> f64 {
        let recent_tests = self.test_history.iter().rev().take(n);
        let (sum, count) = recent_tests.fold((0.0, 0), |(sum, count), entry| {
            (sum + entry.duration_ms, count + 1)
        });
        if count == 0 {
            0.0
        } else {
            sum / count as f64
        }
    }

    pub fn get_recent_success_rate(&self, n: usize) -> f64 {
        let recent_tests: Vec<_> = self.test_history.iter().rev().take(n).collect();
        if recent_tests.is_empty() {
            return 0.0;
        }
        let successes = recent_tests.iter().filter(|entry| entry.success).count();
        successes as f64 / recent_tests.len() as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserHistory {
    pub character_stats: HashMap<String, CharacterStats>,
    pub last_session: DateTime<Utc>,
    pub total_practice_time: f64,
}

impl Default for UserHistory {
    fn default() -> Self {
        Self {
            character_stats: HashMap::new(),
            last_session: Utc::now(),
            total_practice_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KanaType {
    Hiragana,
    Katakana,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PracticeMode {
    Main,
    Dakuten,
    Combination,
    All,
}

impl fmt::Display for KanaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KanaType::Hiragana => write!(f, "hiragana"),
            KanaType::Katakana => write!(f, "katakana"),
        }
    }
}

impl fmt::Display for PracticeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PracticeMode::Main => write!(f, "main"),
            PracticeMode::Dakuten => write!(f, "dakuten"),
            PracticeMode::Combination => write!(f, "combination"),
            PracticeMode::All => write!(f, "all"),
        }
    }
}

impl KanaType {
    pub fn get_practice_set_size(self, mode: PracticeMode) -> usize {
        match (self, mode) {
            (KanaType::Hiragana, PracticeMode::Main) => MAIN_HIRAGANA.len(),
            (KanaType::Hiragana, PracticeMode::Dakuten) => DAKUTEN_HIRAGANA.len(),
            (KanaType::Hiragana, PracticeMode::Combination) => COMBINATION_HIRAGANA.len(),
            (KanaType::Hiragana, PracticeMode::All) => ALL_HIRAGANA.len(),
            (KanaType::Katakana, PracticeMode::Main) => MAIN_KATAKANA.len(),
            (KanaType::Katakana, PracticeMode::Dakuten) => DAKUTEN_KATAKANA.len(),
            (KanaType::Katakana, PracticeMode::Combination) => COMBINATION_KATAKANA.len(),
            (KanaType::Katakana, PracticeMode::All) => ALL_KATAKANA.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Initial, // First start, waiting for Enter
    Ready,   // Showing kana, waiting for input
    Paused,  // User entered empty string, waiting for Enter
}

#[derive(Debug)]
pub struct AppState {
    pub mode: AppMode,
    pub practice_mode: PracticeMode,
    pub kana_type: KanaType,
    pub history: UserHistory,
    pub current_kana: Option<String>,
    pub input_buffer: String,
    pub start_time: Option<DateTime<Utc>>,
    pub expected_romaji: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::Initial,
            practice_mode: PracticeMode::Main,
            kana_type: KanaType::Hiragana,
            history: UserHistory::default(),
            current_kana: None,
            input_buffer: String::new(),
            start_time: None,
            expected_romaji: None,
        }
    }
}
