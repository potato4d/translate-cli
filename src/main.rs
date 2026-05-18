mod agent;
mod app;
mod cli;
mod config;
mod error;
mod input;
mod lang;
mod output;
mod prompt;
mod wizard;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let code = match app::run(&args) {
        Ok(()) => 0,
        Err(error) => error::render_error(&error),
    };
    std::process::exit(code);
}
