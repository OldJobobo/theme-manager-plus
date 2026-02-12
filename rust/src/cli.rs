use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "theme-manager", version, about = "Theme Manager Plus (Rust)")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Option<Command>,
  #[arg(long, global = true, help = "Print the awww command used for transitions")]
  pub debug_awww: bool,
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
  Preset(PresetArgs),
  Waybar(WaybarArgs),
  Walker(WalkerArgs),
  Starship(StarshipArgs),
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
#[command(about = "Interactive picker with inline search (type to filter, Backspace deletes, Ctrl+u clears).")]
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

#[derive(Parser, Debug)]
pub struct PresetArgs {
  #[command(subcommand)]
  pub command: PresetCommand,
}

#[derive(Subcommand, Debug)]
pub enum PresetCommand {
  Save(PresetSaveArgs),
  Load(PresetLoadArgs),
  List,
  Remove(PresetRemoveArgs),
}

#[derive(Parser, Debug)]
pub struct PresetSaveArgs {
  pub name: String,
  #[arg(long)]
  pub theme: Option<String>,
  #[arg(long, value_name = "MODE|NAME")]
  pub waybar: Option<String>,
  #[arg(long, value_name = "MODE|NAME")]
  pub starship: Option<String>,
}

#[derive(Parser, Debug)]
pub struct PresetLoadArgs {
  pub name: String,
  #[arg(short = 'w', long = "waybar", num_args = 0..=1, value_name = "NAME")]
  pub waybar: Option<Option<String>>,
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct PresetRemoveArgs {
  pub name: String,
}

#[derive(Parser, Debug)]
pub struct WaybarArgs {
  pub mode: String,
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct WalkerArgs {
  pub mode: String,
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct StarshipArgs {
  pub mode: String,
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,
}
