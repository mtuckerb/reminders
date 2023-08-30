
use crate::Todo;
use crate::run_applescript;
use redis::{Commands,RedisError};
use std::path::Path;

pub fn create_todo(todo: &Todo) {
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

pub fn check_cache_or_create(path: &Path, todo: &Todo, callback: fn(&Todo)) {
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
