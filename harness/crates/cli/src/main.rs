use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "opencode-harness")]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(short, long)]
        task: Option<String>,
    },
    Report {
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() {
    let _cli = Cli::parse();
}
