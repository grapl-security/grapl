use std::{
    io::Write,
    process::{
        Command,
        Output,
        Stdio,
    },
};

use color_eyre::eyre::Result;

/// Helpers for the CLI, allowing it to execute the generated code
pub fn execute_python(code: &[u8]) -> Result<Output> {
    let mut py_interpreter = Command::new("python3")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start py_interpreter process");

    {
        let child_stdin = py_interpreter.stdin.as_mut().unwrap();
        child_stdin.write_all(code)?;
    }
    Ok(py_interpreter.wait_with_output()?)
}

/// Helpers for the CLI, allowing it to typehceck the generated code
pub fn execute_mypy(code: &str) -> Result<Output> {
    let code = code.replace("'", "\'");
    let py_interpreter = Command::new("mypy")
        .arg("-c")
        .arg(&code)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start py_interpreter process");

    Ok(py_interpreter.wait_with_output()?)
}

pub fn validate_code(code: &str) -> Result<()> {
    let output = execute_python(code.as_bytes())?;
    assert!(output.status.success());

    let output = execute_mypy(code)?;
    assert!(output.status.success());
    Ok(())
}
