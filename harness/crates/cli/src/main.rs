use clap::{Parser, Subcommand};
use opencode_core::loaders::DefaultTaskLoader;
use opencode_core::runners::DifferentialRunner;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opencode-harness")]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(short, long, help = "Path to task file or directory containing tasks")]
        task: Option<String>,
    },
    Report {
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn run_task(task_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let loader = DefaultTaskLoader::new();
    let runner = DifferentialRunner::new(loader);

    let path = PathBuf::from(task_path);

    if path.is_dir() {
        let results = runner.execute_from_path(&path)?;
        println!("Executed {} tasks from directory:", results.len());
        for result in &results {
            println!(
                "  {}: exit_code={}, assertions_passed={}",
                result.task_id, result.exit_code, result.assertions_passed
            );
        }
    } else if path.is_file() {
        let result = runner.execute_single(&path)?;
        println!("Task {} executed:", result.task_id);
        println!("  exit_code: {}", result.exit_code);
        println!("  stdout: {}", result.stdout);
        println!("  stderr: {}", result.stderr);
        println!("  assertions_passed: {}", result.assertions_passed);
    } else {
        eprintln!(
            "Error: Path '{}' is neither a file nor a directory",
            task_path
        );
        std::process::exit(1);
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { task }) => {
            let task_path = task.unwrap_or_else(|| ".".to_string());
            if let Err(e) = run_task(&task_path) {
                eprintln!("Error running task: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Report { output: _ }) => {
            println!("Report command not yet implemented");
        }
        None => {
            println!("No command specified. Use --help for usage information.");
        }
    }
}
