use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
  let cli = theme_manager_plus::cli::Cli::parse();
  if let Err(err) = theme_manager_plus::run(cli) {
    eprintln!("theme-manager: {err}");
    std::process::exit(1);
  }
  Ok(())
}
