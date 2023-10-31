# Fuzzing with Frida

## Fuzz `tauri_cmd_*`

Available commands for testing is `tauri_cmd_1` and `tauri_cmd_2`
- go to directory `src-tauri/frida-fuzz`
- run `cargo r --bin tauri_cmd_*`
    - `tauri_cmd_1` could run for a long time before finding a crash

## Check solutions

- Fuzzing solutions are available in `tauri_cmd_*_solutions`
- check solutions with the files in: `tauri_cmd_*_solutions/file_name`
    - for `tauri_cmd_1` it should contain the value `abc`
    - for `tauri_cmd_2` it should contain the value `100` 
        - with `xxd tauri_cmd_2_solutions/file_name` it should contain `0x0064`

