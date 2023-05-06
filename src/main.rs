use blang::Config;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("error: problem parsing arguments: {err}");
        std::process::exit(1);
    });

    if let Err(e) = blang::run(&config) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
