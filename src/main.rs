use core::panic;
use std::{env, io};
use std::collections::HashMap;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use clap::{arg, ArgAction, command, Command, value_parser};

use persistence::load_tasks;
use persistence::save_tasks;
use task_model::Task;
use utils::format_duration;
use utils::parse_time_string_to_seconds;

mod persistence;
mod task_model;
mod utils;

fn list_tasks(tasks: &HashMap<u32, Task>, list_all: bool, list_timestamp: bool) {
    let mut ids: Vec<u32> = tasks
        .values()
        .filter(|x| x.running || list_all)
        .map(|x| x.id)
        .collect();
    ids.sort();

    if ids.is_empty() {
        println!(
            "There are no {}tasks.",
            if list_all { String::from("") } else { String::from("running ") }
        );
        return;
    }

    let mut total_duration_tasks = 0;
    for id in ids {
        let task = tasks.get(&id).unwrap();
        let duration = get_current_duration(task);
        total_duration_tasks += duration;
        print_task_short(&task, list_timestamp);
    }
    let formatted_total_duration = format_duration(total_duration_tasks);
    println!("\nTotal: {formatted_total_duration}");
}

fn print_task_short(task: &Task, list_timestamp: bool) {
    let duration = get_current_duration(task);
    let formatted_duration = format_duration(duration);
    let prefix = if task.running { "#" } else { "" };
    let timestamp: String = if list_timestamp && task.last_run.is_some() {
        let last_run = task.last_run.unwrap();
        let datetime: DateTime<Local> = last_run.into();
        format!(
            "- Last time: {}",
            datetime.format("%d/%m/%Y %T").to_string()
        )
    } else {
        String::new()
    };
    println!(
        "{}[{}] '{}': {} {}",
        prefix, task.id, task.name, formatted_duration, timestamp
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

fn create_task(tasks: &mut HashMap<u32, Task>, task_name: &str, start: bool) {
    let id = find_new_unique_id(&tasks);
    let task = Task {
        id,
        name: String::from(task_name),
        total_duration_seconds: 0,
        running: start,
        last_run: if start { Some(SystemTime::now()) } else { None },
    };
    tasks.insert(id, task);
    println!("Task {} created with id {}", task_name, id);
}

fn find_new_unique_id(tasks: &HashMap<u32, Task>) -> u32 {
    tasks.keys()
        .map(|k| *k)
        .max()
        .unwrap_or_default() + 1
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
        let additional_time = parse_time_string_to_seconds(time).unwrap();
        task.total_duration_seconds = task.total_duration_seconds + additional_time;
        let duration_formatted = format_duration(task.total_duration_seconds);
        println!("Added {time} to task with id {task_id}, new timer: {duration_formatted}");
    } else {
        println!("Task with id {task_id} does not exist");
    }
}

fn subtract_time(tasks: &mut HashMap<u32, Task>, task_id: &u32, time: &str) {
    if let Some(task) = tasks.get_mut(&task_id) {
        let subtract_time = parse_time_string_to_seconds(time).unwrap();
        if task.total_duration_seconds < subtract_time {
            println!("Task {task_id} does not have enough time to subtract");
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
        let new_total_duration = parse_time_string_to_seconds(time).unwrap();
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
                )
                .arg(
                    arg!(-l --lasttime "List the last time tasks ran")
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
        .subcommand(Command::new("clear").about("Clear all tasks of the selected task type"))
        .get_matches();

    let task_type = match matches.get_one::<String>("tasktype").unwrap().as_str() {
        "current" => "current",
        "archive" => "archive",
        _ => panic!("Could not find a valid task type (current, archive)"),
    };

    let mut tasks = load_tasks(&task_type);

    if let Some(list_matches) = matches.subcommand_matches("list") {
        let list_all = list_matches.get_flag("all");
        let list_timestamp = list_matches.get_flag("lasttime");
        list_tasks(&tasks, list_all, list_timestamp);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_all_tasks() {
        let mut tasks = HashMap::new();
        tasks.insert(1, Task {
            id: 1,
            name: String::from("my task"),
            total_duration_seconds: 0,
            running: false,
            last_run: None,
        });
        list_tasks(&tasks, true, false)
    }

    #[test]
    fn unique_id() {
        let mut tasks = HashMap::new();
        tasks.insert(1, Task {
            id: 1,
            name: String::from("my task"),
            total_duration_seconds: 0,
            running: false,
            last_run: None,
        });
        let id = find_new_unique_id(&tasks);
        assert_eq!(id, 2);
    }

    #[test]
    fn unique_id_empty_tasks() {
        let tasks = HashMap::new();
        let id = find_new_unique_id(&tasks);
        assert_eq!(id, 1);
    }
}
