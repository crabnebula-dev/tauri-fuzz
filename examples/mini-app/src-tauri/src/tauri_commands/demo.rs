/// Tauri commands that can be fuzzed for a demo
use std::fs::File;
use std::io::Read;
/// This command has a backdoor that reads a secret file if the given input is 100
#[tauri::command]
pub fn tauri_cmd_with_backdoor(input: u32) -> String {
    // Normal function
    {
        // println!("Doing computation stuff...")
    }

    // Backdoor left by a malicious developer that run under certain conditions
    if input == 100 {
        // Tries to read a secret file
        let mut content = String::new();
        let mut file = File::open("secret_file.txt").unwrap();
        file.read_to_string(&mut content).unwrap();

        // Send secret over network
        {
            println!("Sending secret content to my server")
        }
    }
    "Finished computations".into()
}

use mysql::{prelude::Queryable, Pool};
/// Tauri command that is vulnerable to SQL injection
/// This finds a student information in the database given it's email address
#[tauri::command]
pub fn sql_injection_vulnerability(input: &str) -> String {
    // We assume that student name will be taken as input
    log::debug!("[sql_transaction] Entering with input: {}", input);

    let url = "mysql://root@localhost/SchoolDatabase";

    // Create MySQL pool
    let pool = Pool::new(url).unwrap();

    // Acquire connection from pool
    let mut conn = pool.get_conn().unwrap();

    // Example query to select all data from Students table
    let query = format!("SELECT * FROM Students where email='{input}'");

    // Execute query
    let students: Vec<(i128, String, String, String, String)> = conn.query(query).unwrap();

    if students.is_empty() {
        println!("Sorry, the student was not found!");
    } else {
        // Iterate over query results
        for result_row in students {
            // Print the data
            println!(
                "Student ID: {}, Name: {} {}, Date of Birth: {}, Email: {}",
                result_row.0, result_row.1, result_row.2, result_row.3, result_row.4
            );
        }
    }
    format!("Hello, you wrote {}!", input)
}
