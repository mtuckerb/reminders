use chrono::{Utc, DateTime};
use crate::Regex;

pub fn get_date_from_string(string: &str) -> Option<String> {
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

pub fn the_day_before(date_string:  &str) -> String {
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


