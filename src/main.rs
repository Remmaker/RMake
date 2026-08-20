mod cli;
mod config;
mod run;
mod build;
mod cache;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("error: {err}");
        // TODO: maybe pass the exit status here
        std::process::exit(1);
    }
}
