# Simple Task Timer

A simple timer for keeping track of your tasks without leaving the terminal.

## Installation

1. Go to [releases page](https://github.com/zambrinf/simple-task-timer/releases) and find the latest release.
2. Download the zip file corresponding to your operating system.
3. Unzip the file to extract the executable. (if you have any issues with Windows try building from source)
4. Place the executable in a folder where it can create a file to store the tasks (make sure you have write permissions for this folder).
5. Add this folder to your PATH.
6. Open a new terminal and run `timer --help`.

You should see the message below:
```
Usage: timer [OPTIONS] [COMMAND]

Commands:
  list     List saved total time of current running tasks added to time elapsed from when it started running
  cancel   Cancel the running of a task without adding the elapsed time to the timer
  create   Create a new task
  delete   Delete a task
  delname  Delete a task by name
  start    Start running a task timer
  stop     Stop running a task timer
  rename   Rename a task
  add      Add time to a task
  sub      Subtract time from a task
  set      Set the total duration time for a task
  archive  Move a task to archive file
  clear    Clear all tasks of the selected task type
  help     Print this message or the help of the given subcommand(s)

Options:
  -t, --tasktype <VALUE>  Type of tasks you want to work on (current, archive) [default: current]
  -h, --help              Print help
  -V, --version           Print version
```

## Examples

List all tasks using `-a` option, currently running tasks are marked with `#`
and they are updated to add the time from when it started running

```
$ timer list -a
[1] 'working-on-my-app': 45:30:00
#[2] 'code-review-pr-x': 00:20:02

Total: 45:50:02
```

Create a task and start running its timer

```
$ timer create -s working-on-my-app
Task working-on-my-app created with id 1
```

Start a task timer

```
$ timer start 1
Task 1 started
```

Stop a task timer

```
$ timer stop 1
Task 1 stopped
```

Add time to a task

```
$ timer add 1 5m
Added 5m to task with id 1, new timer: 23:35:02
```

List running tasks:

```
$ timer list
#[1] 'working-on-my-app': 23:35:02

Total: 23:35:02
```

Set a new time for a task

```
$ timer set 1 45h30m
New time 45h30m set for task 1


$ timer list -a
[1] 'working-on-my-app': 45:30:00
[2] 'code-review-pr-x': 00:20:00

Total: 45:50:00
```

Archive task moves it to archived file which can be accesed using `-t archive`
option

```
$ timer archive 2
Task 2 archived with archive id 1


$ timer list -a
[1] 'working-on-my-app': 45:30:00

Total: 45:30:00


$ timer -t archive list -a
[1] 'code-review-pr-x': 00:20:00

Total: 00:20:00
```

Delete all tasks from archive.

```
$ timer -t archive clear
Do you want to proceed clearing all archive tasks? (Y/N)
y
Tasks cleared.


$ timer -t archive list -a
There are no tasks.
```

## Build from source

```
// Linux
cargo build --release x86_64-unknown-linux-gnu

// Windows
cargo build --release --target x86_64-pc-windows-gnu
```
