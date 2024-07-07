use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::task_model::Task;

fn get_app_folder_path() -> PathBuf {
    let mut app_path = env::current_exe().expect("Could not get the current executable path");
    app_path.pop(); // remove the executable name from the path
    return app_path;
}

pub fn save_tasks(task_type: &str, tasks: &HashMap<u32, Task>) {
    let app_folder_path = get_app_folder_path();
    let mut json_file = String::from(task_type);
    json_file.push_str(".json");
    let current_tasks_path: PathBuf = app_folder_path.join(json_file);
    let mut json_path = File::create(current_tasks_path).unwrap();
    let task_string_json = serde_json::to_string_pretty(&tasks).unwrap();
    json_path.write_all(task_string_json.as_bytes()).unwrap();
}

pub fn load_tasks(task_type: &str) -> HashMap<u32, Task> {
    let app_folder_path = get_app_folder_path();
    let mut json_file = String::from(task_type);
    json_file.push_str(".json");
    let current_tasks_path: PathBuf = app_folder_path.join(json_file);
    if !current_tasks_path.exists() {
        return HashMap::new();
    }
    let mut tasks_file = File::open(&current_tasks_path).unwrap();
    let mut contents = String::new();
    tasks_file.read_to_string(&mut contents).unwrap();
    serde_json::from_str(&contents).unwrap()
}