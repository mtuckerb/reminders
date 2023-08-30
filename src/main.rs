use std::{path::Path, time::Duration};
use notify::{self, RecursiveMode};
use notify_debouncer_mini::{new_debouncer_opt, Config};
use std::process::Command;
use std::path::PathBuf;
use redis::{Commands,RedisError};
use clap::Parser;
use regex::Regex;
use chrono::{Utc, DateTime};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to recursively watch
    #[arg(short, long)]
    path: String,
}

#[derive(Default, Clone, Debug)]
struct Todo {
    name: String,
    due_date: String,
    remind_on: String,
    body: String,
}


fn main() {
    let args = Args::parse();
    let (tx, rx) = std::sync::mpsc::channel();
    let backend_config = notify::Config::default(); //.with_poll_interval(Duration::from_secs(1));
    let debouncer_config = Config::default()
        .with_timeout(Duration::from_millis(1000))
        .with_notify_config(backend_config);
    let mut debouncer = new_debouncer_opt::<_, notify::PollWatcher>(debouncer_config, tx,)
        .unwrap();

    debouncer
        .watcher()
        .watch(Path::new(&args.path), RecursiveMode::Recursive)
        .unwrap();
    for result in rx {
        match result {
            Ok(event) => {
                for e in event {
                    if let Some(ext) = e.path.extension() {
                        if ext == "md" {
                            parse_file(e.path);
                        }
                    }
                }
            },
            Err(error) => println!("Error {error:?}"),
        }
    }
    panic!();
}

fn parse_file(file_path: PathBuf) {
    let mut todo = Todo{..Default::default()};
    let contents = std::fs::read_to_string(&file_path).unwrap();
    for line in contents.lines() {
        if line.starts_with("- [ ] ") {
            // Create a new todo with the contents of the line
            // Replace spaces with underscores to make the name valid
            todo.name = line.replace("- [ ] ", "");
            match get_date_from_string(&todo.name) {
                Some(due_date) => {
                    todo.due_date = due_date.clone();
                    todo.remind_on =  the_day_before(&due_date);
                },
                None => {
                    println!("Couldn't parse date from {}", &todo.name);
                },
            }
            check_cache_or_create(&file_path, &todo, create_todo);
        }
    } 
}

fn create_todo(todo: &Todo) {
    fn format_todo(todo: &Todo) -> String {
        let todo_script = format!(r#"
            on run
            set dueDate to date "{}"
            set remindMe to date "{}"
            tell application "Reminders"
            tell list "Reminders" of default account
            make new reminder with properties {{ name: "{}", due date: dueDate, remind me date: remindMe, body: "{}" }}
            end tell
            end tell
            end run"#, todo.due_date, todo.remind_on, todo.name, todo.body);
        todo_script.to_string()
    }   
    println!("writing todo: {:#?} \n from {:#?}",format_todo(todo), todo);
    run_applescript(format_todo(todo))
}

fn run_applescript(cmd: String) {
    Command::new("osascript")
        .arg("-l")
        .arg("AppleScript")
        .arg("-e")
        .arg(&cmd)
        .output()
        .unwrap();
}

fn check_cache_or_create(path: &Path, todo: &Todo, callback: fn(&Todo)) {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    let string_path = match path.to_str() {
        Some(p) => p,
        None => panic!("could not coerce PathBuf into a String"),
    };
    let check_cache: i8 = con.sismember(string_path, &todo.name).unwrap();
    if check_cache == 1 {
    } else {
        callback(todo);
        let _result: Result<i8, RedisError> =  con.sadd(string_path, &todo.name);
    }
}

fn get_date_from_string(string: &str) -> Option<String> {
    let re = Regex::new(r".?\(@(\d{4}-\d{2}-\d{2}?)\)").unwrap();
    if let Some(captured) = re.captures(string) {
        let mut content = captured.get(1)?.as_str().to_owned(); 
        if content.contains('T') {
        } else {
            content = format!("{}T00:00:00Z", content);
        }
        let date = content.parse::<DateTime<Utc>>(); 

        match date  {
            Ok(date) => {
                return Some(date.format("%A, %B %e, %Y %l:%M:%S %p").to_string().to_owned());
            },
            Err(..) => {
                None
            }
        }
    } else { 
        None 
    }
}

#[test]
fn test_get_date_from_string() {
    let tests = vec![
        ("(@2023-01-01) Test task", Some("Sunday, January  1, 2023 12:00:00 AM".to_owned())),
        ("Test task", None),
        ("@not-a-date Test task", None),
    ];

    for (input, expected_output) in tests {
        let actual_output = get_date_from_string(&input.to_string());
        assert_eq!(actual_output, expected_output);
    }
}

fn the_day_before(date_string:  &str) -> String {
    let to_parse: String = if date_string.contains('T') {
        date_string.to_owned()
    } else { 
        format!("{}T00:00:00Z", date_string)
    };
    let date = to_parse.parse::<DateTime<Utc>>();
    match date  {
        Ok(mut date) => {
            let one_day = chrono::Duration::days(1);
            date -= one_day;
            date.format("%A, %B %e, %Y %l:%M:%S %p").to_string().to_owned()
        },
        Err(..) => {
            date_string.to_owned()
        }
    }
}

#[test]
fn test_the_day_before() {
    let tests = vec![
        ("2023-08-29", "Monday, August 28, 2023 12:00:00 AM"),
        ("2023-08-29T00:00:00Z", "Monday, August 28, 2023 12:00:00 AM")
    ];
    for (input, expected_output) in tests {
        let actual_output = the_day_before(input);
        assert_eq!(actual_output, expected_output);
    }
}


