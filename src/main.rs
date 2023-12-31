use clap::{arg, command, value_parser, ArgAction, Command};
use core::panic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::SystemTime;
use std::{env, io};

#[derive(Serialize, Deserialize)]
struct Task {
    id: u32,
    name: String,
    total_duration_seconds: u64,
    running: bool,
    last_run: Option<SystemTime>,
}

fn get_app_folder_path() -> PathBuf {
    let mut app_path = env::current_exe().expect("Could get the current executable path");
    app_path.pop();
    return app_path;
}

fn save_tasks(task_type: &str, tasks: &HashMap<u32, Task>) {
    let app_folder_path = get_app_folder_path();
    let mut json_file = String::from(task_type);
    json_file.push_str(".json");
    let current_tasks_path: PathBuf = app_folder_path.join(json_file);
    let mut json_path = File::create(current_tasks_path).unwrap();
    let task_string_json = serde_json::to_string_pretty(&tasks).unwrap();
    json_path.write_all(task_string_json.as_bytes()).unwrap();
}

fn load_tasks(task_type: &str) -> HashMap<u32, Task> {
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

fn list_tasks(tasks: &HashMap<u32, Task>, list_all: bool) {
    let mut ids: Vec<u32> = tasks
        .values()
        .filter(|x| x.running || list_all)
        .map(|x| x.id)
        .collect();
    ids.sort();

    if ids.is_empty() {
        println!(
            "There are no {}tasks.",
            if list_all {
                String::from("")
            } else {
                String::from("running ")
            }
        );
        return;
    }

    let mut total_duration_tasks = 0;
    for id in ids {
        let task = tasks.get(&id).unwrap();
        let duration = get_current_duration(task);
        total_duration_tasks += duration;
        print_task_short(&task);
    }
    let formatted_total_duration = format_duration(total_duration_tasks);
    println!("\nTotal: {formatted_total_duration}");
}

fn print_task_short(task: &Task) {
    let duration = get_current_duration(task);
    let formatted_duration = format_duration(duration);
    let prefix = if task.running { "#" } else { "" };
    println!(
        "{}[{}] '{}': {}",
        prefix, task.id, task.name, formatted_duration
    );
}

fn get_current_duration(task: &Task) -> u64 {
    let mut duration = task.total_duration_seconds;
    if task.running {
        let last_run = task.last_run.unwrap();
        let current_running_duration = last_run.elapsed().unwrap_or_default().as_secs();
        duration += current_running_duration;
    }
    duration
}

fn format_duration(duration_seconds: u64) -> String {
    let seconds = duration_seconds % 60;
    let minutes = (duration_seconds / 60) % 60;
    let hours = duration_seconds / 3600;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn create_task(tasks: &mut HashMap<u32, Task>, task_name: &str, start: bool) {
    let id = find_new_unique_id(&tasks);
    let task = Task {
        id: id,
        name: String::from(task_name),
        total_duration_seconds: 0,
        running: start,
        last_run: if start { Some(SystemTime::now()) } else { None },
    };
    tasks.insert(id, task);
    println!("Task {} created with id {}", task_name, id);
}

fn find_new_unique_id(tasks: &HashMap<u32, Task>) -> u32 {
    let mut max_id = 0;
    for task in tasks.values() {
        if task.id > max_id {
            max_id = task.id;
        }
    }
    max_id + 1
}

fn delete_task(tasks: &mut HashMap<u32, Task>, task_id: &u32) {
    if let Some(_task) = tasks.get(&task_id) {
        tasks.remove(&task_id);
        println!("Task {task_id} deleted");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn start_task(tasks: &mut HashMap<u32, Task>, task_id: &u32) {
    if let Some(task) = tasks.get_mut(&task_id) {
        if task.running {
            println!("Task {task_id} is already running");
            return;
        }
        task.running = true;
        task.last_run = Some(SystemTime::now());
        println!("Task {task_id} started");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn stop_task(tasks: &mut HashMap<u32, Task>, task_id: &u32) {
    if let Some(task) = tasks.get_mut(&task_id) {
        if !task.running {
            println!("Task {task_id} is not currently running");
            return;
        }
        task.running = false;
        let mut duration = task.total_duration_seconds;
        let last_run = task.last_run.unwrap();
        let current_running_duration = last_run.elapsed().unwrap_or_default().as_secs();
        duration += current_running_duration;
        task.total_duration_seconds = duration;
        println!("Task {task_id} stopped");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn rename_task(tasks: &mut HashMap<u32, Task>, task_id: &u32, task_name: &str) {
    if let Some(task) = tasks.get_mut(&task_id) {
        task.name = String::from(task_name);
        println!("Task {task_id} rename to {task_name}");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn add_time(tasks: &mut HashMap<u32, Task>, task_id: &u32, time: &str) {
    if let Some(task) = tasks.get_mut(&task_id) {
        let additional_time = parse_time_to_seconds(time).expect("Error parsing the time");
        task.total_duration_seconds = task.total_duration_seconds + additional_time;
        let duration_formatted = format_duration(task.total_duration_seconds);
        println!("Added {time} to task with id {task_id}, new timer: {duration_formatted}");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn parse_time_to_seconds(input: &str) -> Option<u64> {
    let has_hours = input.contains('h');
    let has_minutes = input.contains('m');
    if !has_hours && !has_minutes {
        return None;
    }
    let mut hours: u64 = 0;
    let mut minutes: u64 = 0;
    if has_hours {
        let h_index = input.find('h').unwrap();
        let split_at = input.split_at(h_index);
        hours += String::from(split_at.0)
            .parse::<u64>()
            .expect("could not extract hours");
    }
    if has_minutes {
        let m_index = input.find('m').unwrap();
        let h_index = match input.find('h') {
            Some(index) => index + 1,
            None => 0,
        };
        let slice = &input[h_index..m_index];
        minutes += String::from(slice)
            .parse::<u64>()
            .expect("could not extract hours");
    }
    Some(hours * 3600 + minutes * 60)
}

fn subtract_time(tasks: &mut HashMap<u32, Task>, task_id: &u32, time: &str) {
    if let Some(task) = tasks.get_mut(&task_id) {
        let subtract_time = parse_time_to_seconds(time).expect("Error parsing the time");
        if task.total_duration_seconds < subtract_time {
            println!("Task {task_id} does not have enought time to subtract");
            return;
        }
        task.total_duration_seconds = task.total_duration_seconds - subtract_time;
        let duration_formatted = format_duration(task.total_duration_seconds);
        println!("Subtracted {time} from task {task_id}, new timer: {duration_formatted}");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn set_time(tasks: &mut HashMap<u32, Task>, task_id: &u32, time: &str) {
    if let Some(task) = tasks.get_mut(&task_id) {
        if task.running {
            println!("Task {task_id} is currently running, stop it before setting a new time.");
            return;
        }
        let new_total_duration = parse_time_to_seconds(time).expect("Error parsing the time");
        task.total_duration_seconds = new_total_duration;
        println!("New time {time} set for task {task_id}");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn archive_task(tasks: &mut HashMap<u32, Task>, task_id: &u32) {
    if let Some(task) = tasks.get_mut(&task_id) {
        if task.running {
            println!("Task {task_id} is currently running, stop it before archiving.");
            return;
        }
        let mut archived_tasks = load_tasks("archive");
        let id = find_new_unique_id(&archived_tasks);
        let arch_task = Task {
            id,
            name: task.name.clone(),
            total_duration_seconds: task.total_duration_seconds.clone(),
            running: task.running.clone(),
            last_run: task.last_run.clone(),
        };
        archived_tasks.insert(id, arch_task);
        tasks.remove(task_id);
        save_tasks("archive", &archived_tasks);
        println!("Task {task_id} archived with archive id {id}");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn clear_tasks(tasks: &mut HashMap<u32, Task>, task_type: &str) {
    let mut input = String::new();
    loop {
        println!("Do you want to proceed clearing all {task_type} tasks? (Y/N)");
        if let Err(_) = io::stdin().read_line(&mut input) {
            println!("Error reading input.");
            continue;
        }
        let response = input.trim().to_lowercase();
        if response.eq_ignore_ascii_case("y") {
            break;
        } else if response.eq_ignore_ascii_case("n") {
            println!("Clearing canceled.");
            return;
        } else {
            println!("Invalid input. Please enter 'Y' or 'N'.");
        }
    }
    tasks.clear();
    println!("Tasks cleared.");
}

fn get_task_id_arg(start_matches: &clap::ArgMatches) -> u32 {
    let task_id = start_matches
        .get_one::<String>("task_id")
        .expect("Id required")
        .parse::<u32>()
        .expect("Could not parse the id");
    task_id
}

fn main() {
    let matches = command!()
        .arg(
            arg!(-t --tasktype <VALUE> "Type of tasks you want to work on (current, archive)")
                .value_parser(value_parser!(String))
                .default_value("current"),
        )
        .subcommand(
            Command::new("list")
                .about("List saved total time of current running tasks added to time elapsed from when it started running")
                .arg(
                    arg!(-a --all "List all tasks total time")
                        .required(false)
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("create")
                .about("Create a new task")
                .arg(arg!([name] "Task name").required(true))
                .arg(
                    arg!(-s --start "Start the timer after creating the task")
                        .required(false)
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("delete")
                .about("Delete a task")
                .arg(arg!([task_id] "Task id").required(true)),
        )
        .subcommand(
            Command::new("start")
                .about("Start running a task timer")
                .arg(arg!([task_id] "Task id").required(true)),
        )
        .subcommand(
            Command::new("stop")
                .about("Stop running a task timer")
                .arg(arg!([task_id] "Task id").required(true)),
        )
        .subcommand(
            Command::new("rename")
                .about("Rename a task")
                .arg(arg!([task_id] "Task id").required(true))
                .arg(arg!([name] "Task name").required(true)),
        )
        .subcommand(
            Command::new("add")
                .about("Add time to a task")
                .arg(arg!([task_id] "Task id").required(true))
                .arg(arg!([time] "Time in XXhYYm format. Example: 1h30m").required(true)),
        )
        .subcommand(
            Command::new("sub")
                .about("Subtract time from a task")
                .arg(arg!([task_id] "Task id").required(true))
                .arg(arg!([time] "Time in XXhYYm format. Example: 1h30m").required(true)),
        )
        .subcommand(
            Command::new("set")
                .about("Set the total duration time for a task")
                .arg(arg!([task_id] "Task id").required(true))
                .arg(arg!([time] "Time in XXhYYm format. Example: 1h30m").required(true)),
        )
        .subcommand(
            Command::new("archive")
                .about("Move a task to archive file")
                .arg(arg!([task_id] "Task id").required(true)),
        )
        .subcommand(Command::new("clear").about("Clear all tasks of the selected tasktype"))
        .get_matches();

    let task_type = match matches.get_one::<String>("tasktype").unwrap().as_str() {
        "current" => "current",
        "archive" => "archive",
        _ => panic!("Could not find a valid task type (current, archive)"),
    };

    let mut tasks = load_tasks(&task_type);

    if let Some(list_matches) = matches.subcommand_matches("list") {
        let list_all = list_matches.get_flag("all");
        list_tasks(&tasks, list_all);
    } else if let Some(create_matches) = matches.subcommand_matches("create") {
        let start = create_matches.get_flag("start");
        let task_name = create_matches
            .get_one::<String>("name")
            .expect("Name required");
        create_task(&mut tasks, task_name, start);
    } else if let Some(delete_matches) = matches.subcommand_matches("delete") {
        let task_id = get_task_id_arg(delete_matches);
        delete_task(&mut tasks, &task_id);
    } else if let Some(start_matches) = matches.subcommand_matches("start") {
        let task_id = get_task_id_arg(start_matches);
        start_task(&mut tasks, &task_id);
    } else if let Some(stop_matches) = matches.subcommand_matches("stop") {
        let task_id = get_task_id_arg(stop_matches);
        stop_task(&mut tasks, &task_id);
    } else if let Some(rename_matches) = matches.subcommand_matches("rename") {
        let task_id = get_task_id_arg(rename_matches);
        let task_name = rename_matches
            .get_one::<String>("name")
            .expect("Name required");
        rename_task(&mut tasks, &task_id, task_name);
    } else if let Some(add_matches) = matches.subcommand_matches("add") {
        let task_id = get_task_id_arg(add_matches);
        let time: &String = add_matches
            .get_one::<String>("time")
            .expect("Time required");
        add_time(&mut tasks, &task_id, time);
    } else if let Some(sub_matches) = matches.subcommand_matches("sub") {
        let task_id = get_task_id_arg(sub_matches);
        let time: &String = sub_matches
            .get_one::<String>("time")
            .expect("Time required");
        subtract_time(&mut tasks, &task_id, time);
    } else if let Some(set_matches) = matches.subcommand_matches("set") {
        let task_id = get_task_id_arg(set_matches);
        let time: &String = set_matches
            .get_one::<String>("time")
            .expect("Time required");
        set_time(&mut tasks, &task_id, time);
    } else if let Some(archive_matches) = matches.subcommand_matches("archive") {
        if task_type == "archive" {
            println!("Cannot archive archived tasks");
            return;
        }
        let task_id = get_task_id_arg(archive_matches);
        archive_task(&mut tasks, &task_id);
    } else if let Some(_clear_matches) = matches.subcommand_matches("clear") {
        clear_tasks(&mut tasks, task_type);
    }

    save_tasks(task_type, &tasks);
}
