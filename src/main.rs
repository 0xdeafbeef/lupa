mod adapters;
mod cli;
mod model;
mod render;
mod walk;

fn main() -> std::process::ExitCode {
    match cli::run(std::env::args_os()) {
        Ok(output) => {
            if let Err(err) = write_stdout(&output) {
                let _ = write_stderr(&format!("lupa: failed to write stdout: {err}\n"));
                return std::process::ExitCode::FAILURE;
            }
            std::process::ExitCode::SUCCESS
        }
        Err(err) => {
            let _ = write_stderr(&format!("lupa: {err}\n"));
            std::process::ExitCode::FAILURE
        }
    }
}

fn write_stdout(output: &str) -> std::io::Result<()> {
    use std::io::Write as _;

    let mut stdout = std::io::stdout().lock();
    stdout.write_all(output.as_bytes())
}

fn write_stderr(output: &str) -> std::io::Result<()> {
    use std::io::Write as _;

    let mut stderr = std::io::stderr().lock();
    stderr.write_all(output.as_bytes())
}
