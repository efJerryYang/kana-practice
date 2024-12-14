use std::collections::HashMap;

use crate::error::{Result, KanaError};
use crate::types::*;
use chrono::{DateTime, Utc};
use ratatui::layout::Alignment;
use ratatui::widgets::Axis;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Chart, Dataset, GraphType},
    Frame,
};
use rand::distributions::{Distribution, WeightedIndex};

pub struct App {
    pub state: AppState,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            should_quit: false,
        }
    }

    pub fn handle_enter(&mut self) -> Result<()> {
        match self.state.mode {
            AppMode::Initial | AppMode::Paused => {
                self.state.mode = AppMode::Ready;
                self.select_next_kana()?;
            },
            AppMode::Ready => {
                if self.state.input_buffer.trim().is_empty() {
                    self.state.mode = AppMode::Paused;
                    self.state.current_kana = None;
                    self.state.start_time = None;
                    self.state.input_buffer.clear();
                } else {
                    self.check_answer()?;
                }
            }
        }
        Ok(())
    }

    pub fn select_next_kana(&mut self) -> Result<()> {
        let kana_set: &[(&str, &str)] = match self.state.practice_mode {
            PracticeMode::Main => MAIN_KANA,
            PracticeMode::Dakuten => DAKUTEN_KANA,
            PracticeMode::Combination => COMBINATION_KANA,
            PracticeMode::All => ALL_KANA,
        };
    
        let now = Utc::now();
        
        // Calculate weights for each character
        let weights: Vec<f64> = kana_set
            .iter()
            .map(|&(kana, _)| {
                let stats = self.state.history.character_stats
                    .entry(kana.to_string())
                    .or_insert_with(CharacterStats::new);
                
                stats.calculate_weight(now)
            })
            .collect();
    
        // Create weighted distribution
        let dist = WeightedIndex::new(&weights).map_err(|e| KanaError::Terminal(e.to_string()))?;
        let mut rng = rand::thread_rng();
        
        // Select a character based on weights
        let selected_idx = dist.sample(&mut rng);
        let selected_kana = kana_set[selected_idx];
    
        self.state.current_kana = Some(selected_kana.0.to_string());
        self.state.expected_romaji = Some(selected_kana.1.to_string());
        self.state.start_time = Some(now);
    
        Ok(())
    }

    pub fn handle_input(&mut self, c: char) {
        self.state.input_buffer.push(c);
    }

    pub fn check_answer(&mut self) -> Result<bool> {
        if self.state.mode != AppMode::Ready {
            return Ok(false);
        }

        if let (Some(ref expected), Some(start_time)) = (
            self.state.expected_romaji.as_ref(),
            self.state.start_time
        ) {
            let response_time = (Utc::now() - start_time).num_milliseconds() as f64;
            let input = self.state.input_buffer.trim().to_lowercase();
            let success = input == expected.to_lowercase();

            if let Some(kana) = self.state.current_kana.as_ref() {
                let stats = self.state.history.character_stats
                    .entry(kana.to_string())
                    .or_insert_with(CharacterStats::new);
                
                stats.record_attempt(&input, success, response_time);
            }

            self.state.input_buffer.clear();
            if success {
                self.select_next_kana()?;
            }

            Ok(success)
        } else {
            Ok(false)
        }
    }

    pub fn render(&self, f: &mut Frame) {
        // Use percentage-based constraints for responsive layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),  // Current kana display
                Constraint::Percentage(10),  // User input field
                Constraint::Percentage(38),  // Learning progress graph
                Constraint::Percentage(37),  // Character statistics
                Constraint::Percentage(5),   // Help information
            ])
            .split(f.area());
    
        self.render_kana(f, main_chunks[0]);
        self.render_input(f, main_chunks[1]);
        self.render_learning_progress(f, main_chunks[2]);
        self.render_character_stats_split(f, main_chunks[3]);
        self.render_help(f, main_chunks[4]);
    }
    
    fn render_kana(&self, f: &mut Frame, area: Rect) {
        let kana_text = match self.state.mode {
            AppMode::Initial => "Press Enter to start",
            AppMode::Paused => "Press Enter to continue",
            AppMode::Ready => self.state.current_kana
                .as_ref()
                .map(String::as_str)
                .unwrap_or("Loading..."),
        };
        
        let block = Block::default()
            .title("Current Kana")
            .borders(Borders::ALL);
            
        let paragraph = Paragraph::new(Line::from(vec![
            Span::styled(kana_text, Style::default().fg(Color::Cyan))
        ]))
        .block(block)
        .alignment(Alignment::Center)  // Center horizontally
        .style(Style::default().add_modifier(Modifier::BOLD)); // Make text bold
                
        f.render_widget(paragraph, area);
    }
    
    fn render_input(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Input")
            .borders(Borders::ALL);
            
        let input = Paragraph::new(Line::from(vec![
            Span::raw(&self.state.input_buffer)
        ]))
        .block(block)
        .alignment(Alignment::Center);  // Center horizontally
            
        f.render_widget(input, area);
    }
    
    fn render_learning_progress(&self, f: &mut Frame, area: Rect) {
        // Collect all test entries and sort by timestamp
        let mut all_tests: Vec<(&DateTime<Utc>, f64)> = self.state.history.character_stats.values()
            .flat_map(|stats| &stats.test_history)
            .map(|entry| (&entry.start_time, entry.duration_ms))
            .collect();
        all_tests.sort_by_key(|(time, _)| *time);

        // Calculate EMA for each point
        const ALPHA: f64 = 0.2; // Same alpha as in CharacterStats
        let mut ema_points: Vec<(f64, f64)> = Vec::new();
        let mut ema = 0.0;

        for (idx, (_, duration)) in all_tests.iter().enumerate() {
            if idx == 0 {
                ema = *duration;
            } else {
                ema = ALPHA * duration + (1.0 - ALPHA) * ema;
            }
            ema_points.push((idx as f64, ema));
        }

        if ema_points.is_empty() {
            return;
        }

        // Calculate min/max for better scaling
        let mean = ema_points.iter().map(|(_, v)| *v).sum::<f64>() / ema_points.len() as f64;
        let y_min = (mean * 0.5).max(0.0);
        let y_max = mean * 1.5;
        let y_step = (y_max - y_min) / 5.0;

        // Generate axis labels
        let y_labels: Vec<Span> = (0..=5)
            .map(|i| {
                let value = y_min + y_step * i as f64;
                Span::from(format!("{:.0}ms", value))
            })
            .collect();

        let x_min = 0.0;
        let x_max = ema_points.len() as f64;
        let x_step = x_max / 5.0;

        let x_labels: Vec<Span> = (0..=5)
            .map(|i| {
                let value = x_min + x_step * i as f64;
                Span::from(format!("{:.0}", value))
            })
            .collect();

        // Create dataset with smaller dots
        let dataset = Dataset::default()
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&ema_points);

        let chart = Chart::new(vec![dataset])
            .block(Block::default()
                .title("Response Time Trend (EMA)")
                .borders(Borders::ALL))
            .x_axis(
                Axis::default()
                    .title("Practice Count")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([x_min, x_max])
                    .labels(x_labels)
            )
            .y_axis(
                Axis::default()
                    .title("Response Time (ms)")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([y_min, y_max])
                    .labels(y_labels)
            );

        f.render_widget(chart, area);
    }

    fn render_character_stats_split(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        // Use EMA for both response times and accuracy
        let mut recent_stats: Vec<(&String, f64, f64, usize)> = self.state.history.character_stats
            .iter()
            .map(|(kana, stats)| {
                let ema_accuracy = stats.get_ema_accuracy();
                let ema_response = stats.exp_avg_response;
                let total_tests = stats.test_history.len();
                (kana, ema_accuracy, ema_response, total_tests)
            })
            .collect();

        // Sort by EMA accuracy for the correctness column
        recent_stats.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let correctness_text = Self::render_stats_column(
            "Recent Accuracy (EMA)",
            &recent_stats[..recent_stats.len().min(15)],
            true
        );
        f.render_widget(
            Paragraph::new(correctness_text)
                .block(Block::default().title("By Accuracy").borders(Borders::ALL)),
            chunks[0]
        );

        // Sort by EMA response time for the speed column
        recent_stats.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        let time_text = Self::render_stats_column(
            "Recent Response (EMA)",
            &recent_stats[..recent_stats.len().min(15)],
            false
        );
        f.render_widget(
            Paragraph::new(time_text)
                .block(Block::default().title("By Speed").borders(Borders::ALL)),
            chunks[1]
        );

        let mistakes_text = self.render_mistakes_column();
        f.render_widget(
            Paragraph::new(mistakes_text)
                .block(Block::default().title("Recent Mistakes").borders(Borders::ALL)),
            chunks[2]
        );
    }

    fn render_mistakes_column(&self) -> Vec<Line> {
        let mut text = vec![
            Line::from(vec![
                Span::styled("Recent Mistakes", Style::default().add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
        ];
    
        // Create a romaji to kana lookup map
        let romaji_to_kana: HashMap<&str, &str> = MAIN_KANA.iter()
            .chain(DAKUTEN_KANA.iter())
            .chain(COMBINATION_KANA.iter())
            .map(|&(k, r)| (r, k))
            .collect();
    
        // Group mistakes by kana and track the most recent timestamp
        let mut mistakes_info: Vec<(String, Vec<String>, DateTime<Utc>)> = 
            self.state.history.character_stats.iter()
                .filter(|(_, stats)| !stats.mistakes.is_empty())
                .map(|(kana, stats)| {
                    let wrong_kanas: Vec<String> = stats.mistakes.iter()
                        .filter_map(|m| {
                            romaji_to_kana.get(m.input.as_str())
                                .map(|k| k.to_string()).or_else(|| Some(m.input.clone()))
                        })
                        .collect();
                    
                    // Get the most recent mistake timestamp
                    let latest_timestamp = stats.mistakes.iter()
                        .map(|m| m.timestamp)
                        .max()
                        .unwrap_or_else(|| Utc::now());
    
                    (kana.clone(), wrong_kanas, latest_timestamp)
                })
                .collect();
    
        // Sort by timestamp (most recent first)
        mistakes_info.sort_by(|a, b| b.2.cmp(&a.2));
    
        // Display mistakes
        for (kana, mistakes, _) in mistakes_info.iter().take(15) {
            text.push(Line::from(vec![
                Span::raw(format!("{} → {}", kana, mistakes.join(", ")))
            ]));
        }
    
        text
    }
    
    fn render_stats_column<'a>(
        title: &'a str,
        stats: &'a [(&'a String, f64, f64, usize)],
        is_accuracy: bool,
    ) -> Vec<Line<'a>> {
        let mut text = vec![
            Line::from(vec![
                Span::styled(title, Style::default().add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
        ];

        for (kana, ema_accuracy, ema_response_time, attempts) in stats.iter() {
            let display_value = if is_accuracy {
                format!("{:.1}%", ema_accuracy * 100.0)
            } else {
                format!("{:.0}ms", ema_response_time)
            };

            let value_color = if is_accuracy {
                if *ema_accuracy < 0.8 { Color::Red }
                else if *ema_accuracy < 0.9 { Color::Yellow }
                else { Color::Green }
            } else {
                if *ema_response_time > 2000.0 { Color::Red }
                else if *ema_response_time > 1000.0 { Color::Yellow }
                else { Color::Green }
            };

            text.push(Line::from(vec![
                Span::raw(format!("{}: ", kana)),
                Span::styled(display_value, Style::default().fg(value_color)),
                Span::raw(format!(" ({} tests)", attempts)),
            ]));
        }

        text
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = Line::from(vec![
            Span::raw("ESC to quit | Enter to submit | Type romaji for the shown kana")
        ]);
        
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL));
            
        f.render_widget(help, area);
    }
}