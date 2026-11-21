#[cfg(unix)]
mod native;

fn main() -> std::process::ExitCode {
    #[cfg(unix)]
    let status = match native::run() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("gino: {}", error.to_string().escape_debug());
            1
        }
    };
    #[cfg(not(unix))]
    let status = {
        eprintln!("gino: this native adapter currently supports Unix terminals only");
        2
    };
    std::process::ExitCode::from(status)
}
