// // Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use mysql::{prelude::Queryable, Pool, PooledConn};

/// Create a Database with some data first
/// ```sql
/// -- Create the database
/// CREATE DATABASE SchoolDatabase;

/// -- Switch to the newly created database
/// USE SchoolDatabase;

/// -- Create the Students table
/// CREATE TABLE Students (
///     student_id INT PRIMARY KEY AUTO_INCREMENT,
///     first_name VARCHAR(50),
///     last_name VARCHAR(50),
///     date_of_birth DATE,
///     email VARCHAR(100)
/// );

/// -- Insert data into the Students table
/// INSERT INTO Students (first_name, last_name, date_of_birth, email) VALUES
/// ('John', 'Doe', '2000-05-10', 'john.doe@example.com'),
/// ('Jane', 'Smith', '2001-08-15', 'jane.smith@example.com'),
/// ('Michael', 'Johnson', '1999-12-20', 'michael.johnson@example.com'),
/// ('Emily', 'Brown', '2002-03-25', 'emily.brown@example.com'),
/// ('Daniel', 'Martinez', '2000-11-05', 'daniel.martinez@example.com');
/// ```

const SQL_DB: &str = "mysql://root@localhost/SchoolDatabase";
fn connect_to_db() -> Result<PooledConn, mysql::Error> {
    let pool = Pool::new(SQL_DB)?;
    pool.get_conn()
}

#[tauri::command]
/// Crash on input `abc`
pub fn sql_transaction(input: &str) -> String {
    // We assume that student name will be taken as input
    tracing::debug!("[sql_transaction] Entering with input: {}", input);

    let mut conn = connect_to_db().unwrap();

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
