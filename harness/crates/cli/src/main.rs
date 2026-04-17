use clap::{Parser, Subcommand};
use opencode_core::loaders::DefaultTaskLoader;
use opencode_core::logging::init_logger;
use opencode_core::runners::DifferentialRunner;
use std::path::PathBuf;
use tracing::{error, info};

mod report;
use report::ReportCommand;

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
        info!("Executed {} tasks from directory:", results.len());
        for result in &results {
            info!("  {}: verdict={:?}", result.task_id, result.verdict);
        }
    } else if path.is_file() {
        let result = runner.execute_single(&path)?;
        info!("Task {} executed:", result.task_id);
        info!("  verdict: {:?}", result.verdict);
        info!("  duration_ms: {}", result.duration_ms);
        if let Some(ref legacy) = result.legacy_result {
            info!("  legacy exit_code: {:?}", legacy.exit_code);
        }
        if let Some(ref rust_res) = result.rust_result {
            info!("  rust exit_code: {:?}", rust_res.exit_code);
        }
    } else {
        error!(
            "Error: Path '{}' is neither a file nor a directory",
            task_path
        );
        std::process::exit(1);
    }

    Ok(())
}

fn main() {
    init_logger();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { task }) => {
            let task_path = task.unwrap_or_else(|| ".".to_string());
            if let Err(e) = run_task(&task_path) {
                error!("Error running task: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Report { output }) => {
            let output_format = output.unwrap_or_else(|| "json".to_string());
            match ReportCommand::execute(&output_format) {
                Ok(()) => {}
                Err(e) => {
                    error!("Report error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            info!("No command specified. Use --help for usage information.");
        }
    }
}
