lazy_static! {
    pub static ref ARGS: Cli = Cli::parse();
}
use serde::{Deserialize, Serialize};
//static LOGGER:Logger= logger::configure_log();
use clap::Parser;

#[derive(Parser, Debug, Clone, Default)]
#[command(version)]
pub struct Cli {
    /// path to the  configuration file (default: if not specified, settings.json file in same directory as executable is used)
    #[arg(short, long)]
    pub config: Option<String>,
    /// The path to store the data.json file (default is same directory as executable)
    #[arg(short, long)]
    pub data_dir: Option<String>,
    /// shown more logging information, default is true,
    #[arg(short, long)]
    pub verbose: bool,
}
