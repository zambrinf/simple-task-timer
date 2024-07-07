use std::time::SystemTime;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::utils::{format_duration, parse_time_string_to_seconds};

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub total_duration_seconds: u64,
    pub running: bool,
    pub last_run: Option<SystemTime>,
}

impl Task {
    pub fn to_print_string(&self, list_timestamp: bool) -> String {
        let duration = self.current_duration();
        let formatted_duration = format_duration(duration);
        let prefix = if self.running { "#" } else { "" };
        let timestamp: String = if list_timestamp && self.last_run.is_some() {
            let last_run = self.last_run.unwrap();
            let datetime: DateTime<Local> = last_run.into();
            format!(
                "- Last time: {}",
                datetime.format("%d/%m/%Y %T").to_string()
            )
        } else {
            String::new()
        };
        format!(
            "{}[{}] '{}': {} {}",
            prefix, self.id, self.name, formatted_duration, timestamp
        )
    }

    pub fn current_duration(&self) -> u64 {
        let mut duration = self.total_duration_seconds;
        if self.running {
            let last_run = self.last_run.unwrap();
            let current_running_duration = last_run.elapsed().unwrap_or_default().as_secs();
            duration += current_running_duration;
        }
        duration
    }

    pub fn start(&mut self) -> Result<(), ()> {
        if self.running {
            return Err(());
        }
        self.running = true;
        self.last_run = Some(SystemTime::now());
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), ()> {
        if !self.running {
            return Err(());
        }
        let current_duration = self.total_duration_seconds;
        let last_run = self.last_run.unwrap();
        let duration_since_last_run = last_run.elapsed().unwrap_or_default().as_secs();
        self.total_duration_seconds = current_duration + duration_since_last_run;
        self.running = false;
        Ok(())
    }

    pub fn rename(&mut self, task_name: &str) {
        self.name = String::from(task_name);
    }

    pub fn set_time(&mut self, time: &str) -> Result<(), ()> {
        if self.running {
            return Err(());
        }
        let new_duration = parse_time_string_to_seconds(time).unwrap();
        self.total_duration_seconds = new_duration;
        Ok(())
    }

    pub fn add_time(&mut self, time: &str) {
        let additional_time = parse_time_string_to_seconds(time).unwrap();
        self.total_duration_seconds = self.total_duration_seconds + additional_time;
    }

    pub fn subtract_time(&mut self, time: &str) -> Result<(), ()> {
        let subtract_time = parse_time_string_to_seconds(time).unwrap();
        if self.total_duration_seconds < subtract_time {
            return Err(());
        }
        self.total_duration_seconds = self.total_duration_seconds - subtract_time;
        Ok(())
    }

    pub fn formatted_duration(&self) -> String {
        format_duration(self.total_duration_seconds)
    }
}
