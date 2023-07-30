use clap::{Parser, Subcommand};
use ztr::ztr;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Zettle
    Create {},
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Create {}) => {
            let name = ztr::create(&std::path::PathBuf::from(r#"/Users/meysi/notebox"#));
            print!("{}", name.to_string_lossy())
        }
        None => {
            print!("No subcommand was used");
        }
    }
}
