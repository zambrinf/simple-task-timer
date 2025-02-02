use core::panic;
use std::{env, io};
use std::collections::HashMap;
use std::time::SystemTime;

use clap::{arg, ArgAction, ArgMatches, command, Command, value_parser};

use persistence::load_tasks;
use persistence::save_tasks;
use task::Task;
use utils::format_duration;

mod persistence;
mod task;
mod utils;

fn list_tasks(tasks: &HashMap<u32, Task>, list_all: bool, show_timestamp: bool, show_base_timer: bool) {
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
        let duration = task.current_duration();
        total_duration_tasks += duration;
        println!("{}", task.to_print_string(show_timestamp, show_base_timer))
    }
    let formatted_total_duration = format_duration(total_duration_tasks);
    println!("\nTotal: {formatted_total_duration}");
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

fn delete_task_by_id(tasks: &mut HashMap<u32, Task>, task_id: &u32) {
    if tasks.contains_key(task_id) {
        tasks.remove(&task_id);
        println!("Task {task_id} deleted");
    } else {
        task_does_not_exist(task_id);
    }
}

fn delete_task_by_name(tasks: &mut HashMap<u32, Task>, task_name: &str) {
    let filter: Vec<&Task> = tasks.values()
        .filter(|t| t.name.eq(task_name))
        .collect();
    if filter.len() > 1 {
        println!("Error: more than one task with name {task_name}.");
    } else {
        let task_opt = filter.last();
        match task_opt {
            None => {
                task_does_not_exist_name(task_name);
            }
            Some(task) => {
                tasks.remove(&task.id.clone());
                println!("Task {task_name} deleted");
            }
        }
    }
}

fn start_task(task: &mut Task) {
    let res = task.start();
    match res {
        Ok(_) => {
            println!("Task {} started", task.id);
        }
        Err(_) => {
            println!("Task {} is already running", task.id);
        }
    }
}

fn stop_task(task: &mut Task) {
    let res = task.stop();
    match res {
        Ok(_) => {
            println!("Task {} stopped", task.id);
        }
        Err(_) => {
            println!("Task {} is not currently running", task.id);
        }
    }
}

fn cancel_task(task: &mut Task) {
    let res = task.cancel();
    match res {
        Ok(_) => {
            println!("Task {} cancelled", task.id);
        }
        Err(_) => {
            println!("Task {} is not currently running", task.id);
        }
    }
}

fn rename_task(task: &mut Task, task_name: &str) {
    task.rename(task_name);
    println!("Task {} renamed to {}", task.id, task_name);
}

fn task_does_not_exist(task_id: &u32) {
    println!("Task with id {task_id} does not exist");
}

fn task_does_not_exist_name(task_name: &str) {
    println!("Task with name {task_name} does not exist");
}

fn add_time(task: &mut Task, time: &str) {
    task.add_time(time);
    let duration_formatted = task.formatted_duration();
    println!("Added {time} to task with id {}, new timer: {duration_formatted}", task.id);
}

fn subtract_time(task: &mut Task, time: &str) {
    let res = task.subtract_time(time);
    match res {
        Ok(_) => {
            let duration_formatted = task.formatted_duration();
            println!("Subtracted {time} from task {}, new timer: {duration_formatted}", task.id);
        }
        Err(_) => {
            println!("Task {} does not have enough time to subtract", task.id);
        }
    }
}

fn set_time(task: &mut Task, time: &str) {
    let res = task.set_time(time);
    match res {
        Ok(_) => {
            println!("New time {time} set for task {}", task.id);
        }
        Err(_) => {
            println!("Task {} is currently running, stop it before setting a new time.", task.id);
        }
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
        task_does_not_exist(task_id);
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

fn get_task<'a>(tasks: &'a mut HashMap<u32, Task>, task_id: &u32) -> Result<&'a mut Task, ()> {
    return if let Some(task) = tasks.get_mut(&task_id) {
        Ok(task)
    } else {
        Err(())
    };
}

fn get_task_id_arg(start_matches: &ArgMatches) -> u32 {
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
                    arg!(-l --lasttime "Show the last time tasks ran")
                        .required(false)
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    arg!(-b --base "Show the base timer, without adding running elapsed time")
                        .required(false)
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("cancel")
                .about("Cancel a running task, returning to its previous state")
                .arg(arg!([task_id] "Task id").required(true)),
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
            Command::new("delname")
                .about("Delete a task by name")
                .arg(arg!([name] "Task name").required(true)),
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
        let show_timestamp = list_matches.get_flag("lasttime");
        let show_base_timer = list_matches.get_flag("base");
        list_tasks(&tasks, list_all, show_timestamp, show_base_timer);
    } else if let Some(start_matches) = matches.subcommand_matches("cancel") {
        let task_id = get_task_id_arg(start_matches);
        let task_res = get_task(&mut tasks, &task_id);
        if let Ok(task) = task_res {
            cancel_task(task);
        } else {
            task_does_not_exist(&task_id);
        }
    } else if let Some(create_matches) = matches.subcommand_matches("create") {
        let start = create_matches.get_flag("start");
        let task_name = create_matches
            .get_one::<String>("name")
            .expect("Name required");
        create_task(&mut tasks, task_name, start);
    } else if let Some(delete_matches) = matches.subcommand_matches("delete") {
        let task_id = get_task_id_arg(delete_matches);
        delete_task_by_id(&mut tasks, &task_id);
    } else if let Some(delete_name_matches) = matches.subcommand_matches("delname") {
        let task_name = delete_name_matches
            .get_one::<String>("name")
            .expect("Name required");
        delete_task_by_name(&mut tasks, &task_name);
    } else if let Some(start_matches) = matches.subcommand_matches("start") {
        let task_id = get_task_id_arg(start_matches);
        let task_res = get_task(&mut tasks, &task_id);
        if let Ok(task) = task_res {
            start_task(task);
        } else {
            task_does_not_exist(&task_id);
        }
    } else if let Some(stop_matches) = matches.subcommand_matches("stop") {
        let task_id = get_task_id_arg(stop_matches);
        let task_res = get_task(&mut tasks, &task_id);
        if let Ok(task) = task_res {
            stop_task(task);
        } else {
            task_does_not_exist(&task_id);
        }
    } else if let Some(rename_matches) = matches.subcommand_matches("rename") {
        let task_id = get_task_id_arg(rename_matches);
        let task_name = rename_matches
            .get_one::<String>("name")
            .expect("Name required");
        let task_res = get_task(&mut tasks, &task_id);
        if let Ok(task) = task_res {
            rename_task(task, task_name);
        } else {
            task_does_not_exist(&task_id);
        }
    } else if let Some(add_matches) = matches.subcommand_matches("add") {
        let task_id = get_task_id_arg(add_matches);
        let time: &String = add_matches
            .get_one::<String>("time")
            .expect("Time required");
        let task_res = get_task(&mut tasks, &task_id);
        if let Ok(task) = task_res {
            add_time(task, time);
        } else {
            task_does_not_exist(&task_id);
        }
    } else if let Some(sub_matches) = matches.subcommand_matches("sub") {
        let task_id = get_task_id_arg(sub_matches);
        let time: &String = sub_matches
            .get_one::<String>("time")
            .expect("Time required");
        let task_res = get_task(&mut tasks, &task_id);
        if let Ok(task) = task_res {
            subtract_time(task, time);
        } else {
            task_does_not_exist(&task_id);
        }
    } else if let Some(set_matches) = matches.subcommand_matches("set") {
        let task_id = get_task_id_arg(set_matches);
        let time: &String = set_matches
            .get_one::<String>("time")
            .expect("Time required");
        let task_res = get_task(&mut tasks, &task_id);
        if let Ok(task) = task_res {
            set_time(task, time);
        } else {
            task_does_not_exist(&task_id);
        }
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
        list_tasks(&tasks, true, false, false);
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
