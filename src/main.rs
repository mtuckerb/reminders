use std::{path::Path, time::Duration};
use notify::{self, RecursiveMode};
use notify_debouncer_mini::{new_debouncer_opt, Config};
use std::process::Command;
use std::path::PathBuf;
use redis::{Commands,RedisError, RedisResult};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to recursively watch
    #[arg(short, long)]
    path: String,
}

fn main() {
    let args = Args::parse();
    let (tx, rx) = std::sync::mpsc::channel();
    let backend_config = notify::Config::default().with_poll_interval(Duration::from_secs(15));
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
                    match e.path.extension() {
                        Some(ext) => {
                            if ext == "md" {parse_file(e.path);}
                        },
                        None => {},
                    };
                }
            },
            Err(error) => println!("Error {error:?}"),
        }
    }
    loop {};
}

fn parse_file(file_path: PathBuf) {
    // Read the file contents
    let contents = std::fs::read_to_string(&file_path).unwrap();

    // Search for TODO lines
    for line in contents.lines() {
        if line.starts_with("- [ ] ") {
            // Create a new todo with the contents of the line
            // Replace spaces with underscores to make the name valid
            let todo = line.replace("- [ ] ", "");
            check_cache_or_create(&file_path, todo, create_todo);
        }
    }
}

fn create_todo(todo: String) {
    fn format_todo(todo: String) -> String {
        let todo_script = format!(r#"
            on run
            tell application "Reminders"
            tell list "Reminders" of default account
            make new reminder with properties {{name: "{}"}}
            end tell
            end tell
            end run"#, todo);
        return todo_script.to_string();
    }   
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

fn check_cache_or_create(path: &PathBuf, todo: String, callback: fn(String)) {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    let string_path = match path.to_str() {
        Some(p) => p,
        None => panic!("could not coerce PathBuf into a String"),
    };

    let check_cache: i8 = con.sismember(string_path, todo.clone()).unwrap();
    if check_cache == 1 {
    } else {
        callback(todo.clone());
        let _result: Result<i8, RedisError> =  con.sadd(string_path, todo.clone());
    }
}
