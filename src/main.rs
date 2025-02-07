mod seneca_client;
mod utils;

use seneca_client::SenecaClient;
use utils::{generate_assignment_string, input_or_clipboard, parse_keys_file, pause};

use chrono::{DateTime, Utc};
use indicatif::ProgressBar;
use inquire::{Confirm, Select, Text};
use regex::Regex;
use serde_json::Value;
use std::error::Error;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (api_key, refresh_token) = match parse_keys_file() {
        Ok(keys) => keys,
        Err(e) => {
            eprintln!("❌ Error reading seneca-solver-keys.json: {}", e);
            eprintln!("This error is likely caused by a nonexistent or invalid seneca-solver-keys.json file. Ensure it is placed in the same folder as this program and/or try re-creating it.");
            pause();
            return Err(e);
        }
    };

    let mut client = match SenecaClient::new(api_key, refresh_token).await {
        Ok(client) => {
            println!("🔑 Successfully authenticated with Seneca");
            client
        }
        Err(e) => {
            eprintln!("❌ Error authenticating with Seneca: {}", e);
            eprintln!("This error is likely caused by an invalid seneca-solver-keys.json file. Try re-creating that file.");
            pause();
            return Err(e);
        }
    };

    loop {
        let result = assignment_loop(&mut client).await;

        if result.is_err() {
            eprintln!("🚨 Error: {}\n", result.err().unwrap());
        }

        let repeat = Confirm::new("Would you like to run the solver again?")
            .with_default(false)
            .prompt();

        match repeat {
            Ok(true) => println!("🔄 Restarting solver\n"),
            Ok(false) => {
                println!("Exiting solver");
                break Ok(());
            }
            Err(_) => return Err("Invalid input".into()),
        }
    }
}

async fn assignment_loop(client: &mut SenecaClient) -> Result<(), Box<dyn Error>> {
    let assignments = client.get_assignments().await;
    let assignments = match assignments {
        Ok(assignments) => {
            println!("📬 Fetched assignments");
            Ok(assignments)
        }
        Err(e) => {
            eprintln!("❌ Error fetching assignments due to an unknown error. This error is not caused by an incorrect access key.
Please report the following error at https://github.com/ArcaEge/seneca-solver/issues");
            Err(e)
        }
    }?;

    // Filter out assignments that have not started yet
    let now = Utc::now();
    let assignments: Vec<&Value> = assignments
        .iter()
        .filter(|assignment| {
            DateTime::parse_from_rfc3339(assignment["startDate"].as_str().unwrap()).unwrap() <= now
        })
        .collect();

    if std::env::args()
        .collect::<Vec<String>>()
        .contains(&"--xp-farm".to_string())
    {
        println!("🚜 Running in XP farm mode");
        loop {
            solve_all_assignments(client, &assignments).await?;
        };
    }

    // Find longest assignment name and longest status to use for padding
    let longest_assignment_length = assignments
        .iter()
        .map(|assignment| assignment["name"].as_str().unwrap().len())
        .max()
        .unwrap_or(0);
    let longest_status_length = assignments
        .iter()
        .map(|assignment| assignment["status"].as_str().unwrap().len())
        .max()
        .unwrap_or(0);

    let assignment_names: Vec<String> = assignments
        .iter()
        .map(|assignment| {
            generate_assignment_string(assignment, longest_assignment_length, longest_status_length)
        })
        .collect();
    let mut assignment_names: Vec<&str> = assignment_names.iter().map(|s| s.as_str()).collect();

    // Other options
    assignment_names.insert(0, "All assignments");
    assignment_names.insert(1, "Custom (from URL)");

    let assignment_name = Select::new("Choose assignment:", assignment_names).prompt()?;

    match assignment_name {
        "Custom (from URL)" => solve_custom(client).await,
        "All assignments" => solve_all_assignments(client, &assignments).await,
        _ => {
            let assignment = assignments
                .iter()
                .find(|assignment| {
                    generate_assignment_string(
                        assignment,
                        longest_assignment_length,
                        longest_status_length,
                    ) == assignment_name
                })
                .unwrap();

            println!(
                "📝 Solving assignment: {}",
                assignment["name"].as_str().unwrap()
            );

            solve_assignments(assignment, client).await
        }
    }
}

async fn solve_assignments(
    assignment: &Value,
    client: &mut SenecaClient,
) -> Result<(), Box<dyn Error>> {
    let course_id = assignment["spec"]["courseId"].as_str().unwrap().to_string();
    let mut section_ids = assignment["spec"]["sectionIds"].as_array().unwrap().clone();

    // Shuffle section IDs
    section_ids.shuffle(&mut thread_rng());

    let section_id_len = section_ids.len();

    let progress_bar = ProgressBar::new(section_id_len as u64);

    for section_id in section_ids {
        let section_id = section_id.as_str().unwrap();
        let (_, _, contents) = client.get_contents(&course_id, section_id).await?;
        let contents = contents.as_array().unwrap();

        for content in contents {
            if let Err(e) = client.run_solver(&course_id, section_id, content).await {
                eprintln!("Error running solver for section {}: {}", section_id, e);
            }
        }

        // Increment progress bar
        progress_bar.inc(1);
    }

    progress_bar.finish();
    println!(
        "⏱️ Assignment solved in {} seconds",
        progress_bar.elapsed().as_secs_f32()
    );

    Ok(())
}

async fn solve_custom(client: &mut SenecaClient) -> Result<(), Box<dyn Error>> {
    let course_id = input_or_clipboard(
        Text::new("Enter URL of section:")
            .with_help_message("Leave blank to use clipboard")
            .prompt(),
    )?;

    let re = Regex::new(r"(?<course_id>\w{8}-\w{4}-\w{4}-\w{4}-\w{12})/section/(?<section_id>\w{8}-\w{4}-\w{4}-\w{4}-\w{12})").unwrap();

    if let Some(captures) = re.captures(&course_id) {
        let course_id = captures.name("course_id").unwrap().as_str();
        let section_id = captures.name("section_id").unwrap().as_str();

        let contents_tuple = client.get_contents(&course_id, &section_id).await;

        // Check if error is 404
        if let Err(e) = &contents_tuple {
            if let Some(reqwest_error) = e.downcast_ref::<reqwest::Error>() {
                if reqwest_error.is_status() {
                    return Err("Course and/or section not found. Double-check that you have entered a valid Seneca URL".into());
                }
            }
        }

        let (index, title, contents) = contents_tuple?;
        let contents = contents.as_array().unwrap();

        println!("📝 Solving section: ");
        println!(
            "🔍 Found {} subsection(s) in section {}: {}",
            contents.len(),
            index,
            title
        );

        if contents.len() > 1 {
            let content_names: Vec<String> = contents
                .iter()
                .map(|content| {
                    content["tags"].as_array().unwrap()[0]
                        .as_str()
                        .unwrap()
                        .to_string()
                })
                .collect();

            let content_name = Select::new("Choose subsection:", content_names).prompt()?;

            let content = contents
                .iter()
                .find(|content| {
                    content["tags"].as_array().unwrap()[0]
                        .as_str()
                        .unwrap()
                        .to_string()
                        == content_name
                })
                .unwrap();

            // for content in contents {
            client.run_solver(&course_id, &section_id, content).await?;
            // }
        } else {
            client
                .run_solver(&course_id, &section_id, &contents[0])
                .await?;
        }

        println!("✅ Subsection solved");

        Ok(())
    } else {
        Err("Invalid assignment URL. This URL should be in the format https://app.senecalearning.com/classroom/course/<course_id>/section/<section_id>/session".into())
    }
}

async fn solve_all_assignments(
    client: &mut SenecaClient,
    assignments: &Vec<&Value>,
) -> Result<(), Box<dyn Error>> {
    let assignments_len = assignments.len();

    println!("🚀 Solving {} assignment(s)", assignments_len);

    for (i, assignment) in assignments.iter().enumerate() {
        println!(
            "📝 Solving assignment: {}     {}/{assignments_len}",
            assignment["name"].as_str().unwrap(),
            i + 1,
        );
        solve_assignments(assignment, client).await?;
    }

    println!("✅ All assignments solved");

    Ok(())
}
