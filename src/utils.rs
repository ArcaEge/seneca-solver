use chrono::{DateTime, Duration, Utc};
use copypasta::{ClipboardContext, ClipboardProvider};
use inquire::InquireError;
use rand::Rng;
use serde_json;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::io;
use std::io::prelude::*;

pub fn read_clipboard() -> String {
    let mut clipboard_context = ClipboardContext::new().unwrap();
    clipboard_context.get_contents().unwrap_or("".to_string())
}

// Generates a 12-character hexadecimal string
pub fn generate_hex_string(half_length: i32) -> String {
    let mut rng = rand::thread_rng();
    (0..half_length)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect::<String>()
}

pub fn generate_assignment_string(
    assignment: &Value,
    longest_assignment_length: usize,
    longest_status_length: usize,
) -> String {
    let name = assignment["name"].as_str().unwrap();
    let due = DateTime::parse_from_rfc3339(assignment["dueDate"].as_str().unwrap())
        .unwrap()
        .to_rfc2822();

    // Remove timezone
    let due = &due[..due.len() - 6];

    // Get status
    let status = assignment["status"].as_str().unwrap();

    // Create padding strings
    let padding_assignment = " ".repeat(longest_assignment_length - name.len());
    let padding_status = " ".repeat(longest_status_length - status.len());

    format!(
        "{}{}   {}{}  Due: {} (UTC)",
        name, padding_assignment, status, padding_status, due
    )
}

pub fn input_or_clipboard(
    user_input: Result<String, InquireError>,
) -> Result<String, Box<dyn Error>> {
    let input = if let Ok(input) = user_input {
        input
    } else {
        println!("");
        return Err("Failed to read input".into());
    };

    // Read input from clipboard if none was entered, then trim whitespace
    Ok(if input.is_empty() {
        read_clipboard()
    } else {
        input
    }
    .trim()
    .to_string())
}

// Generates a random duration within a range
pub fn generate_random_duration(min: Duration, max: Duration) -> Duration {
    let mut rng = rand::thread_rng();
    Duration::seconds(rng.gen_range(min.num_seconds()..max.num_seconds()))
}

pub fn generate_time_vec(
    end: DateTime<Utc>,
    min: Duration,
    max: Duration,
    count: usize,
) -> (DateTime<Utc>, Vec<DateTime<Utc>>) {
    let mut time_vec = vec![];
    let mut start_time = end;

    for _ in 0..count {
        let duration = generate_random_duration(min, max);
        start_time -= duration;
        time_vec.push(start_time);
    }

    time_vec.push(end);

    (start_time, time_vec)
}

pub fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "\nPress enter to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

pub fn parse_keys_file() -> Result<(String, String), Box<dyn Error>> {
    let path = "./seneca-solver-keys.json";
    let data = fs::read_to_string(path)?;
    let res: Value = serde_json::from_str(&data).map_err(|e| -> Box<dyn Error> { e.into() })?;

    let keys = res.as_object().ok_or("Invalid JSON formatting")?;
    let api_key = keys
        .get("apiKey")
        .ok_or("Missing API key in file")?
        .as_str()
        .ok_or("Invalid formatting of API key in file")?
        .to_string();
    let refresh_token = keys
        .get("refreshToken")
        .ok_or("Missing refresh token in file")?
        .as_str()
        .ok_or("Invalid formatting of refresh token in file")?
        .to_string();

    Ok((api_key, refresh_token))
}
