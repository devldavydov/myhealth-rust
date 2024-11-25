use clap::Parser;

#[derive(Parser)]
#[command()]
enum Cli {
    Bot(bot::args::ArgsCli),
}

pub fn parse() -> Box<dyn service::Service> {
    let cli = Cli::parse();
    match cli {
        Cli::Bot(args) => Box::new(bot::app::App::new(args)),
    }
}
