use std::{path::Path, time::Duration};
use notify::{self, RecursiveMode};
use notify_debouncer_mini::{new_debouncer_opt, Config};
use std::process::Command;
use std::path::PathBuf;
use clap::Parser;
use regex::Regex;

mod date_helpers;
use date_helpers::{get_date_from_string,the_day_before};

mod create_todo;
use create_todo::{create_todo, check_cache_or_create};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to recursively watch
    #[arg(short, long)]
    path: String,
}

#[derive(Default, Clone, Debug)]
pub struct Todo {
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


pub fn run_applescript(cmd: String) {
    Command::new("osascript")
        .arg("-l")
        .arg("AppleScript")
        .arg("-e")
        .arg(&cmd)
        .output()
        .unwrap();
}



