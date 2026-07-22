use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
  name = "theme-manager",
  version = env!("THEME_MANAGER_VERSION"),
  about = "Theme Manager Plus (Rust)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[arg(
        long,
        global = true,
        help = "Print the awww command used for transitions"
    )]
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
    Hyprlock(HyprlockArgs),
    Unlock(UnlockArgs),
    Starship(StarshipArgs),
    Bar(BarArgs),
    Plugin(PluginArgs),
}

#[derive(Parser, Debug)]
pub struct SetArgs {
    pub theme: String,
    #[arg(short = 'w', long = "waybar", num_args = 0..=1, value_name = "NAME")]
    pub waybar: Option<Option<String>>,
    #[arg(short = 'k', long = "walker", num_args = 0..=1, value_name = "NAME")]
    pub walker: Option<Option<String>>,
    #[arg(long = "hyprlock", num_args = 0..=1, value_name = "NAME")]
    pub hyprlock: Option<Option<String>>,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct NextArgs {
    #[arg(short = 'w', long = "waybar", num_args = 0..=1, value_name = "NAME")]
    pub waybar: Option<Option<String>>,
    #[arg(short = 'k', long = "walker", num_args = 0..=1, value_name = "NAME")]
    pub walker: Option<Option<String>>,
    #[arg(long = "hyprlock", num_args = 0..=1, value_name = "NAME")]
    pub hyprlock: Option<Option<String>>,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
#[command(
    about = "Interactive picker with inline search (type to filter, Backspace deletes, Ctrl+u clears)."
)]
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
    pub walker: Option<String>,
    #[arg(long, value_name = "MODE|NAME")]
    pub hyprlock: Option<String>,
    #[arg(long, value_name = "MODE|NAME")]
    pub starship: Option<String>,
}

#[derive(Parser, Debug)]
pub struct PresetLoadArgs {
    pub name: String,
    #[arg(short = 'w', long = "waybar", num_args = 0..=1, value_name = "NAME")]
    pub waybar: Option<Option<String>>,
    #[arg(short = 'k', long = "walker", num_args = 0..=1, value_name = "NAME")]
    pub walker: Option<Option<String>>,
    #[arg(long = "hyprlock", num_args = 0..=1, value_name = "NAME")]
    pub hyprlock: Option<Option<String>>,
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
pub struct HyprlockArgs {
    pub mode: String,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct UnlockArgs {
    #[command(subcommand)]
    pub command: UnlockCommand,
}

#[derive(Subcommand, Debug)]
pub enum UnlockCommand {
    List,
    Set(UnlockSetArgs),
    Reset(UnlockResetArgs),
}

#[derive(Parser, Debug)]
pub struct UnlockSetArgs {
    pub theme: String,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct UnlockResetArgs {
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct StarshipArgs {
    pub mode: String,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
#[command(
    about = "Manage the Omarchy 4.0+ Quickshell bar (requires omarchy-shell)"
)]
pub struct BarArgs {
    #[command(subcommand)]
    pub command: BarCommand,
}

#[derive(Subcommand, Debug)]
pub enum BarCommand {
    /// List installed bar options (built-in and third-party)
    List,
    /// Switch the active bar to the given plugin id
    Use(BarUseArgs),
    /// Return to the built-in Omarchy bar
    Reset(BarQuietArgs),
    /// Restore the default bar layout and service widgets
    Defaults(BarQuietArgs),
    /// Set the bar position
    Position(BarPositionArgs),
    /// Set bar transparency
    Transparent(BarTransparentArgs),
    /// Add, move, remove, set, or replace bar widgets in the layout
    /// (passed straight through to `omarchy-bar-plugin` with interactive stdio)
    Plugin(BarPluginArgs),
}

#[derive(Parser, Debug)]
pub struct BarPluginArgs {
    #[command(subcommand)]
    pub command: BarPluginCommand,
}

#[derive(Subcommand, Debug)]
pub enum BarPluginCommand {
    /// Anything (add/move/remove/set/replace) is passed straight through
    /// to `omarchy-bar-plugin` with interactive stdio
    #[command(external_subcommand)]
    Other(Vec<String>),
}

#[derive(Parser, Debug)]
pub struct BarUseArgs {
    pub id: String,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct BarQuietArgs {
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct BarPositionArgs {
    #[arg(value_name = "top|bottom|left|right")]
    pub position: String,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
pub struct BarTransparentArgs {
    #[arg(value_name = "true|false")]
    pub value: String,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

#[derive(Parser, Debug)]
#[command(about = "Manage Omarchy 4.0+ Quickshell plugins (requires omarchy-shell)")]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(Subcommand, Debug)]
pub enum PluginCommand {
    /// List discovered shell plugins
    List,
    /// Enable a plugin by id
    Enable(PluginIdArgs),
    /// Disable a plugin by id
    Disable(PluginIdArgs),
    /// Anything else (add/update/remove/rescan/clone/edit/validate) is passed
    /// straight through to `omarchy-plugin` with interactive stdio
    #[command(external_subcommand)]
    Other(Vec<String>),
}

#[derive(Parser, Debug)]
pub struct PluginIdArgs {
    pub id: String,
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}
