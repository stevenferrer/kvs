use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Set(SetArgs),
    Get(GetArgs),
    Rm(RmArgs),
}

#[derive(Args)]
struct SetArgs {
    #[clap(value_parser)]
    key: String,

    #[clap(value_parser)]
    value: String,
}

#[derive(Args)]
struct GetArgs {
    #[clap(value_parser)]
    key: String,
}

#[derive(Args)]
struct RmArgs {
    #[clap(value_parser)]
    key: String,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Set(_) => {
            panic!("unimplemented")
        }
        Commands::Get(_) => {
            panic!("unimplemented")
        }
        Commands::Rm(_) => {
            panic!("unimplemented")
        }
    }
}
