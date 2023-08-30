[![Rust](https://github.com/mtuckerb/reminders/actions/workflows/rust.yml/badge.svg)](https://github.com/mtuckerb/reminders/actions/workflows/rust.yml)

# Summary
This little app will monitor a directory of your choice for changes. When a notify event occurs, it will check the changed file for markdown style todos.
If one is found, it will create an Apple Reminder. If a date is present (on the same line) in the format `(@2023-01-01)`, 
it will set the due date and remind me date to that value. 

I use this with [Obsidian](https://obsidian.md).


# Install
download a release and `sudo cp reminders /usr/local/bin` 

# Usage
`reminders -p <path_to_monitor> &`
