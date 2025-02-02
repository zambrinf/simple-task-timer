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
    pub fn to_print_string(&self, show_timestamp: bool, show_base_timer: bool) -> String {
        let duration = self.current_duration();
        let formatted_duration = format_duration(duration);
        let prefix = if self.running { "#" } else { "" };
        let timestamp = self.get_timestamp(show_timestamp);
        let base_timer = self.get_base_timer_formatted(show_base_timer);
        format!(
            "{}[{}] '{}': {}{}{}",
            prefix, self.id, self.name, formatted_duration, base_timer, timestamp
        )
    }

    fn get_timestamp(&self, show_timestamp: bool) -> String {
        let timestamp: String = if show_timestamp && self.last_run.is_some() {
            let last_run = self.last_run.unwrap();
            let datetime: DateTime<Local> = last_run.into();
            format!(
                " - Last time: {}",
                datetime.format("%d/%m/%Y %T").to_string()
            )
        } else {
            String::new()
        };
        timestamp
    }

    fn get_base_timer_formatted(&self, show_base_timer: bool) -> String {
        let timestamp: String = if show_base_timer {
            let duration = self.total_duration_seconds;
            let formatted_duration = format_duration(duration);
            format!(
                " - Base timer: {}",
                formatted_duration
            )
        } else {
            String::new()
        };
        timestamp
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

    pub fn cancel(&mut self) -> Result<(), ()> {
        if !self.running {
            return Err(());
        }
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

#[cfg(test)]
mod tests {
    use std::ops::Sub;

    use super::*;

    #[test]
    fn to_print_string_default() {
        let t = Task {
            id: 1,
            name: String::from("my task"),
            total_duration_seconds: 60,
            running: false,
            last_run: None,
        };
        let print_string = t.to_print_string(false, false);
        assert_eq!("[1] 'my task': 00:01:00", print_string);
    }

    #[test]
    fn to_print_string_running() {
        let t = Task {
            id: 1,
            name: String::from("my task"),
            total_duration_seconds: 60,
            running: true,
            last_run: Some(SystemTime::now()),
        };
        let print_string = t.to_print_string(false, false);
        assert_eq!("#[1] 'my task': 00:01:00", print_string);
    }

    #[test]
    fn to_print_string_running_sub() {
        let task = Task {
            id: 1,
            name: String::from("my task"),
            total_duration_seconds: 60,
            running: true,
            last_run: Some(SystemTime::now().sub(std::time::Duration::new(5, 0))),
        };
        let print_string = task.to_print_string(false, false);
        assert_eq!("#[1] 'my task': 00:01:05", print_string);
    }

    #[test]
    fn to_print_string_running_sub_show_base() {
        let task = Task {
            id: 1,
            name: String::from("my task"),
            total_duration_seconds: 60,
            running: true,
            last_run: Some(SystemTime::now().sub(std::time::Duration::new(5, 0))),
        };
        let print_string = task.to_print_string(false, true);
        assert_eq!("#[1] 'my task': 00:01:05 - Base timer: 00:01:00", print_string);
    }

    #[test]
    fn cancel_running_task() {
        let mut task = Task {
            id: 1,
            name: String::from("my task"),
            total_duration_seconds: 60,
            running: true,
            last_run: Some(SystemTime::now().sub(std::time::Duration::new(5, 0))),
        };
        let res = task.cancel();
        assert_eq!(false, res.is_err());
        assert_eq!(false, task.running);
    }

    #[test]
    fn cancel_running_task_error_not_running() {
        let mut task = Task {
            id: 1,
            name: String::from("my task"),
            total_duration_seconds: 60,
            running: false,
            last_run: Some(SystemTime::now().sub(std::time::Duration::new(5, 0))),
        };
        let res = task.cancel();
        assert_eq!(true, res.is_err());
    }
}