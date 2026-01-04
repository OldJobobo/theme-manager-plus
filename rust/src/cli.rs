use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "theme-manager", version, about = "Theme Manager Plus (Rust)")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
  List,
  Set(SetArgs),
  Next(NextArgs),
  Browse(BrowseArgs),
  Current,
  BgNext,
  PrintConfig,
  Version,
  Install(InstallArgs),
  Update,
  Remove(RemoveArgs),
}

#[derive(Parser, Debug)]
pub struct SetArgs {
  pub theme: String,
  #[arg(short = 'w', long = "waybar", num_args = 0..=1, value_name = "NAME")]
  pub waybar: Option<Option<String>>,
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct NextArgs {
  #[arg(short = 'w', long = "waybar", num_args = 0..=1, value_name = "NAME")]
  pub waybar: Option<Option<String>>,
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct BrowseArgs {
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct InstallArgs {
  pub git_url: String,
}

#[derive(Parser, Debug)]
pub struct RemoveArgs {
  pub theme: Option<String>,
}
