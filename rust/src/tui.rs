use anyhow::{anyhow, Result};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::fs;
use std::io::{stdout, Stdout};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;
use ansi_to_tui::IntoText;
use ratatui_core::layout::Alignment as CoreAlignment;
use ratatui_core::style::{Color as CoreColor, Modifier as CoreModifier, Style as CoreStyle};
use ratatui_core::text::{Line as CoreLine, Span as CoreSpan, Text as CoreText};

use crate::config::ResolvedConfig;
use crate::paths::title_case_theme;
use crate::preview;

#[derive(Debug)]
pub struct BrowseSelection {
  pub theme: String,
  pub waybar: WaybarSelection,
  pub starship: StarshipSelection,
}

#[derive(Debug)]
pub enum WaybarSelection {
  UseDefaults,
  None,
  Auto,
  Named(String),
}

#[derive(Debug)]
pub enum StarshipSelection {
  UseDefaults,
  None,
  Preset(String),
  Named(String),
  Theme(PathBuf),
}

struct PreviewBackend {
  kind: PreviewBackendKind,
}

enum PreviewBackendKind {
  Kitty,
  Chafa,
  None,
}

impl PreviewBackend {
  fn detect() -> Self {
    if command_exists("kitty") && (std::env::var("KITTY_WINDOW_ID").is_ok() || term_contains("kitty") || term_contains("ghostty")) {
      return PreviewBackend {
        kind: PreviewBackendKind::Kitty,
      };
    }
    if command_exists("chafa") {
      return PreviewBackend {
        kind: PreviewBackendKind::Chafa,
      };
    }
    PreviewBackend {
      kind: PreviewBackendKind::None,
    }
  }

  fn render(&self, path: Option<&Path>, rect: Rect) {
    match self.kind {
      PreviewBackendKind::Kitty => {
        if let Some(path) = path {
          let place = format!("{}x{}@{}x{}", rect.width, rect.height, rect.x, rect.y);
          let _ = Command::new("kitty")
            .args([
              "+kitten",
              "icat",
              "--clear",
              "--transfer-mode=stream",
              "--stdin=no",
              "--scale-up",
              "--place",
              &place,
              path.to_string_lossy().as_ref(),
            ])
            .status();
        } else {
          let _ = Command::new("kitty")
            .args(["+kitten", "icat", "--clear", "--stdin=no"])
            .status();
        }
      }
      _ => {}
    }
  }

  fn text_preview(&self, path: Option<&Path>, rect: Rect) -> Text<'_> {
    match self.kind {
      PreviewBackendKind::Chafa => {
        if let Some(path) = path {
          let size = format!("{}x{}", rect.width.max(1), rect.height.max(1));
          if let Ok(output) = Command::new("chafa")
            .args(["--format=symbols", "--size", &size, path.to_string_lossy().as_ref()])
            .output()
          {
            if output.status.success() {
              if let Ok(rendered) = String::from_utf8(output.stdout) {
                return Text::from(rendered);
              }
            }
          }
        }
        Text::from("No preview available.")
      }
      _ => {
        if let Some(path) = path {
          Text::from(path.to_string_lossy().to_string())
        } else {
          Text::from("No preview available.")
        }
      }
    }
  }
}

pub fn browse(config: &ResolvedConfig, quiet: bool) -> Result<Option<BrowseSelection>> {
  if quiet {
    // currently unused, but reserved for future use
  }
  let themes = list_theme_entries(&config.theme_root_dir)?;
  if themes.is_empty() {
    return Err(anyhow!("no themes available"));
  }

  let theme_items: Vec<OptionItem> = themes
    .into_iter()
    .map(|name| {
      let label = title_case_theme(&name);
      let theme_path = config.theme_root_dir.join(&name);
      let preview_path = preview::find_theme_preview(&theme_path);
      OptionItem {
        label,
        value: name,
        preview: preview_path,
      }
    })
    .collect();

  let backend = PreviewBackend::detect();
  let selected_theme = match select_list("Select theme", &theme_items, &backend)? {
    Some(index) => theme_items[index].value.clone(),
    None => return Ok(None),
  };

  let theme_path = config.theme_root_dir.join(&selected_theme);

  let waybar_selection = match build_waybar_options(config, &theme_path)? {
    SelectionOptions::UseDefaults => WaybarSelection::UseDefaults,
    SelectionOptions::Items(items) => {
      let choice = match select_list("Select Waybar", &items, &backend)? {
        Some(index) => items[index].clone(),
        None => return Ok(None),
      };
      match choice.kind.as_str() {
        "default" => WaybarSelection::None,
        "theme" => WaybarSelection::Auto,
        _ => WaybarSelection::Named(choice.value),
      }
    }
  };

  let starship_selection = match build_starship_options(config, &theme_path)? {
    StarshipOptions::UseDefaults => StarshipSelection::UseDefaults,
    StarshipOptions::Items(items) => {
      let choice = match select_starship_list("Select Starship", &items, &backend, &theme_path, config)? {
        Some(index) => items[index].clone(),
        None => return Ok(None),
      };
      match choice.kind.as_str() {
        "default" => StarshipSelection::None,
        "theme" => StarshipSelection::Theme(theme_path.join("starship.yaml")),
        "preset" => StarshipSelection::Preset(choice.value),
        _ => StarshipSelection::Named(choice.value),
      }
    }
  };

  Ok(Some(BrowseSelection {
    theme: selected_theme,
    waybar: waybar_selection,
    starship: starship_selection,
  }))
}

struct OptionItem {
  label: String,
  value: String,
  preview: Option<PathBuf>,
}

impl OptionItem {
  fn with_kind(label: String, value: String, kind: &str, preview: Option<PathBuf>) -> LabeledItem {
    LabeledItem {
      label,
      value,
      kind: kind.to_string(),
      preview,
    }
  }
}

#[derive(Clone)]
struct LabeledItem {
  label: String,
  value: String,
  kind: String,
  preview: Option<PathBuf>,
}

enum SelectionOptions {
  UseDefaults,
  Items(Vec<LabeledItem>),
}

enum StarshipOptions {
  UseDefaults,
  Items(Vec<LabeledItem>),
}

fn build_waybar_options(config: &ResolvedConfig, theme_path: &Path) -> Result<SelectionOptions> {
  let mut items = Vec::new();
  items.push(OptionItem::with_kind(
    "Omarchy default".to_string(),
    "default".to_string(),
    "default",
    None,
  ));

  let theme_waybar = theme_path.join("waybar-theme");
  if theme_waybar.join("config.jsonc").is_file() && theme_waybar.join("style.css").is_file() {
    let preview_path = preview::find_waybar_preview(&theme_waybar);
    items.push(OptionItem::with_kind(
      "Use theme waybar".to_string(),
      "theme".to_string(),
      "theme",
      preview_path,
    ));
  }

  for name in list_waybar_themes(&config.waybar_themes_dir)? {
    let preview_path = preview::find_waybar_preview(&config.waybar_themes_dir.join(&name));
    items.push(OptionItem::with_kind(
      name.clone(),
      name,
      "named",
      preview_path,
    ));
  }

  if items.len() <= 1 {
    return Ok(SelectionOptions::UseDefaults);
  }

  Ok(SelectionOptions::Items(items))
}

fn build_starship_options(config: &ResolvedConfig, theme_path: &Path) -> Result<StarshipOptions> {
  let mut items = Vec::new();
  items.push(OptionItem::with_kind(
    "Omarchy default".to_string(),
    "default".to_string(),
    "default",
    None,
  ));

  if theme_path.join("starship.yaml").is_file() {
    items.push(OptionItem::with_kind(
      "Use theme starship".to_string(),
      "theme".to_string(),
      "theme",
      None,
    ));
  }

  for preset in list_starship_presets() {
    items.push(OptionItem::with_kind(
      format!("Preset: {preset}"),
      preset,
      "preset",
      None,
    ));
  }

  for theme in list_starship_themes(&config.starship_themes_dir)? {
    items.push(OptionItem::with_kind(
      format!("Theme: {theme}"),
      theme,
      "named",
      None,
    ));
  }

  if items.len() <= 1 {
    return Ok(StarshipOptions::UseDefaults);
  }

  Ok(StarshipOptions::Items(items))
}

fn select_list(title: &str, items: &[impl ItemView], backend: &PreviewBackend) -> Result<Option<usize>> {
  let mut terminal = setup_terminal()?;
  let mut state = ListState::default();
  state.select(Some(0));
  let mut last_preview = None::<PathBuf>;

  loop {
    terminal.draw(|frame| {
      let size = frame.area();
      let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(size);

      let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| ListItem::new(Line::from(item.label())))
        .collect();
      let list = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
      frame.render_stateful_widget(list, chunks[0], &mut state);

      let preview_path = state
        .selected()
        .and_then(|idx| items.get(idx))
        .and_then(|item| item.preview_path().cloned());

      if preview_path != last_preview {
        backend.render(preview_path.as_deref(), chunks[1]);
        last_preview = preview_path.clone();
      }

      let preview_text = backend.text_preview(preview_path.as_deref(), chunks[1]);
      let preview = Paragraph::new(preview_text)
        .block(Block::default().title("Preview").borders(Borders::ALL));
      frame.render_widget(preview, chunks[1]);
    })?;

    if event::poll(Duration::from_millis(200))? {
      if let Event::Key(key) = event::read()? {
        match key.code {
          KeyCode::Char('q') | KeyCode::Esc => {
            cleanup_terminal(&mut terminal)?;
            return Ok(None);
          }
          KeyCode::Enter => {
            let selected = state.selected().unwrap_or(0);
            cleanup_terminal(&mut terminal)?;
            return Ok(Some(selected));
          }
          KeyCode::Up | KeyCode::Char('k') => {
            let new_index = previous_index(state.selected(), items.len());
            state.select(Some(new_index));
          }
          KeyCode::Down | KeyCode::Char('j') => {
            let new_index = next_index(state.selected(), items.len());
            state.select(Some(new_index));
          }
          KeyCode::Home => state.select(Some(0)),
          KeyCode::End => state.select(Some(items.len().saturating_sub(1))),
          _ => {}
        }
      }
    }
  }
}

fn select_starship_list(
  title: &str,
  items: &[LabeledItem],
  _backend: &PreviewBackend,
  theme_path: &Path,
  config: &ResolvedConfig,
) -> Result<Option<usize>> {
  let mut terminal = setup_terminal()?;
  let mut state = ListState::default();
  state.select(Some(0));
  let mut last_index = None::<usize>;
  let mut last_width = None::<u16>;
  let mut last_text = Text::from("Loading preview...");

  loop {
    terminal.draw(|frame| {
      let size = frame.area();
      let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(size);

      let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| ListItem::new(Line::from(item.label())))
        .collect();
      let list = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
      frame.render_stateful_widget(list, chunks[0], &mut state);

      let selected = state.selected().unwrap_or(0);
      let width = chunks[1].width.max(40);
      if Some(selected) != last_index || Some(width) != last_width {
        let choice = &items[selected];
        last_text = render_starship_preview(choice, theme_path, config, width);
        last_index = Some(selected);
        last_width = Some(width);
      }

      let preview = Paragraph::new(last_text.clone())
        .block(Block::default().title("Preview").borders(Borders::ALL));
      frame.render_widget(preview, chunks[1]);
    })?;

    if event::poll(Duration::from_millis(200))? {
      if let Event::Key(key) = event::read()? {
        match key.code {
          KeyCode::Char('q') | KeyCode::Esc => {
            cleanup_terminal(&mut terminal)?;
            return Ok(None);
          }
          KeyCode::Enter => {
            let selected = state.selected().unwrap_or(0);
            cleanup_terminal(&mut terminal)?;
            return Ok(Some(selected));
          }
          KeyCode::Up | KeyCode::Char('k') => {
            let new_index = previous_index(state.selected(), items.len());
            state.select(Some(new_index));
          }
          KeyCode::Down | KeyCode::Char('j') => {
            let new_index = next_index(state.selected(), items.len());
            state.select(Some(new_index));
          }
          KeyCode::Home => state.select(Some(0)),
          KeyCode::End => state.select(Some(items.len().saturating_sub(1))),
          _ => {}
        }
      }
    }
  }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
  enable_raw_mode()?;
  let mut stdout = stdout();
  execute!(stdout, terminal::EnterAlternateScreen)?;
  let backend = CrosstermBackend::new(stdout);
  Terminal::new(backend).map_err(|err| anyhow!("failed to init terminal: {err}"))
}

fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
  terminal.show_cursor()?;
  Ok(())
}

fn next_index(current: Option<usize>, len: usize) -> usize {
  if len == 0 {
    return 0;
  }
  match current {
    Some(idx) => (idx + 1) % len,
    None => 0,
  }
}

fn previous_index(current: Option<usize>, len: usize) -> usize {
  if len == 0 {
    return 0;
  }
  match current {
    Some(idx) => {
      if idx == 0 {
        len - 1
      } else {
        idx - 1
      }
    }
    None => 0,
  }
}

fn list_theme_entries(theme_root: &Path) -> Result<Vec<String>> {
  if !theme_root.is_dir() {
    return Err(anyhow!(
      "themes directory not found: {}",
      theme_root.to_string_lossy()
    ));
  }
  let mut entries = Vec::new();
  for entry in fs::read_dir(theme_root)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() || is_symlink(&path)? {
      if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        entries.push(name.to_string());
      }
    }
  }
  entries.sort();
  Ok(entries)
}

fn list_waybar_themes(waybar_themes_dir: &Path) -> Result<Vec<String>> {
  if !waybar_themes_dir.is_dir() {
    return Ok(Vec::new());
  }
  let mut entries = Vec::new();
  for entry in fs::read_dir(waybar_themes_dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir()
      && path.join("config.jsonc").is_file()
      && path.join("style.css").is_file()
    {
      if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        entries.push(name.to_string());
      }
    }
  }
  entries.sort();
  Ok(entries)
}

fn list_starship_presets() -> Vec<String> {
  if !command_exists("starship") {
    return Vec::new();
  }
  if let Ok(output) = Command::new("starship").args(["preset", "--list"]).output() {
    if output.status.success() {
      return parse_lines(&output.stdout);
    }
  }
  if let Ok(output) = Command::new("starship").args(["preset", "-l"]).output() {
    if output.status.success() {
      return parse_lines(&output.stdout);
    }
  }
  Vec::new()
}

fn list_starship_themes(dir: &Path) -> Result<Vec<String>> {
  if !dir.is_dir() {
    return Ok(Vec::new());
  }
  let mut themes = Vec::new();
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_file() {
      if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if ext.eq_ignore_ascii_case("toml") {
          if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            themes.push(stem.to_string());
          }
        }
      }
    }
  }
  themes.sort();
  Ok(themes)
}

fn render_starship_preview(
  choice: &LabeledItem,
  theme_path: &Path,
  config: &ResolvedConfig,
  width: u16,
) -> Text<'static> {
  match choice.kind.as_str() {
    "default" => {
      return Text::from("No Starship config change\n\nThe current Omarchy theme prompt will be used.");
    }
    _ => {}
  }

  if !command_exists("starship") {
    return Text::from("Starship not found in PATH.");
  }

  let temp_dir = match TempDir::new() {
    Ok(dir) => dir,
    Err(_) => return Text::from("Failed to create preview temp dir."),
  };
  let preview_root = temp_dir.path();
  let _ = Command::new("git").arg("init").arg("-q").current_dir(preview_root).status();
  let _ = fs::write(preview_root.join("README.md"), "mock");
  let _ = Command::new("git")
    .arg("add")
    .arg(".")
    .current_dir(preview_root)
    .status();

  let config_path = match choice.kind.as_str() {
    "theme" => {
      let path = theme_path.join("starship.yaml");
      if !path.is_file() {
        return Text::from("Theme-specific Starship config not found.");
      }
      path
    }
    "preset" => {
      let preset_name = choice.value.as_str();
      let output = Command::new("starship")
        .args(["preset", preset_name])
        .output();
      let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Text::from(format!("Failed to load preset: {preset_name}")),
      };
      let preset_path = preview_root.join("preset.toml");
      if fs::write(&preset_path, output.stdout).is_err() {
        return Text::from("Failed to write preset file.");
      }
      preset_path
    }
    "named" => {
      let path = config.starship_themes_dir.join(format!("{}.toml", choice.value));
      if !path.is_file() {
        return Text::from(format!(
          "Theme config not found: {}",
          path.to_string_lossy()
        ));
      }
      path
    }
    _ => {
      return Text::from("Unknown selection.");
    }
  };

  let width_str = width.to_string();
  let prompt_output = Command::new("starship")
    .args([
      "prompt",
      "--path",
      preview_root.to_string_lossy().as_ref(),
      "--terminal-width",
      &width_str,
      "--jobs",
      "0",
    ])
    .env("STARSHIP_CONFIG", &config_path)
    .output();

  let prompt = match prompt_output {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).to_string(),
    _ => "Failed to render prompt.".to_string(),
  };

  let right_output = Command::new("starship")
    .args([
      "prompt",
      "--right",
      "--path",
      preview_root.to_string_lossy().as_ref(),
      "--terminal-width",
      &width_str,
    ])
    .env("STARSHIP_CONFIG", &config_path)
    .output();

  let right_prompt = match right_output {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).to_string(),
    _ => String::new(),
  };

  let mut lines: Vec<Line<'static>> = Vec::new();
  lines.push(Line::from("=== Starship Prompt Preview ==="));
  lines.push(Line::from(""));

  let left_lines = trim_empty_lines(parse_ansi_lines(&strip_prompt_markers(&prompt)));
  let right_trimmed = strip_prompt_markers(right_prompt.trim());
  if !right_trimmed.is_empty() {
    let right_lines = trim_empty_lines(parse_ansi_lines(&right_trimmed));
    lines.extend(combine_prompt_lines(&left_lines, &right_lines, width));
  } else {
    lines.extend(left_lines);
  }

  lines.push(Line::from(""));
  lines.push(Line::from("---"));
  lines.push(Line::from(format!("Config: {}", choice.label)));

  Text::from(lines)
}

fn strip_prompt_markers(input: &str) -> String {
  input.replace("\\[", "").replace("\\]", "")
}

fn parse_ansi_lines(input: &str) -> Vec<Line<'static>> {
  match input.as_bytes().into_text() {
    Ok(text) => convert_text(text).lines,
    Err(_) => input
      .lines()
      .map(|line| Line::from(line.to_string()))
      .collect(),
  }
}

fn combine_prompt_lines(
  left_lines: &[Line<'static>],
  right_lines: &[Line<'static>],
  width: u16,
) -> Vec<Line<'static>> {
  if left_lines.is_empty() {
    return right_lines.to_vec();
  }
  if right_lines.is_empty() {
    return left_lines.to_vec();
  }

  let mut out = Vec::new();
  let total_width = width as usize;

  if left_lines.len() > 1 {
    out.extend_from_slice(&left_lines[..left_lines.len() - 1]);
  }

  let left_last = left_lines[left_lines.len() - 1].clone();
  let right_first = right_lines[0].clone();
  let left_width = left_last.width();
  let right_width = right_first.width();
  let spacer_width = total_width.saturating_sub(left_width + right_width);

  let mut spans = left_last.spans;
  if spacer_width > 0 {
    spans.push(ratatui::text::Span::raw(" ".repeat(spacer_width)));
  }
  spans.extend(right_first.spans);
  out.push(Line::from(spans));

  if right_lines.len() > 1 {
    out.extend_from_slice(&right_lines[1..]);
  }
  out
}

fn trim_empty_lines(mut lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
  while lines.first().map(|l| l.width() == 0).unwrap_or(false) {
    lines.remove(0);
  }
  while lines.last().map(|l| l.width() == 0).unwrap_or(false) {
    lines.pop();
  }
  lines
}

fn convert_text(text: CoreText<'static>) -> Text<'static> {
  let lines = text
    .lines
    .into_iter()
    .map(convert_line)
    .collect::<Vec<_>>();
  Text {
    lines,
    style: convert_style(text.style),
    alignment: text.alignment.map(convert_alignment),
  }
}

fn convert_line(line: CoreLine<'static>) -> Line<'static> {
  let spans = line
    .spans
    .into_iter()
    .map(convert_span)
    .collect::<Vec<_>>();
  Line {
    spans,
    style: convert_style(line.style),
    alignment: line.alignment.map(convert_alignment),
  }
}

fn convert_span(span: CoreSpan<'static>) -> ratatui::text::Span<'static> {
  ratatui::text::Span {
    content: span.content,
    style: convert_style(span.style),
  }
}

fn convert_style(style: CoreStyle) -> Style {
  let mut out = Style::default();
  out.fg = style.fg.map(convert_color);
  out.bg = style.bg.map(convert_color);
  out.add_modifier = convert_modifier(style.add_modifier);
  out.sub_modifier = convert_modifier(style.sub_modifier);
  out
}

fn convert_modifier(modifier: CoreModifier) -> Modifier {
  Modifier::from_bits_truncate(modifier.bits())
}

fn convert_alignment(alignment: CoreAlignment) -> ratatui::layout::Alignment {
  match alignment {
    CoreAlignment::Left => ratatui::layout::Alignment::Left,
    CoreAlignment::Center => ratatui::layout::Alignment::Center,
    CoreAlignment::Right => ratatui::layout::Alignment::Right,
  }
}

fn convert_color(color: CoreColor) -> Color {
  match color {
    CoreColor::Reset => Color::Reset,
    CoreColor::Black => Color::Black,
    CoreColor::Red => Color::Red,
    CoreColor::Green => Color::Green,
    CoreColor::Yellow => Color::Yellow,
    CoreColor::Blue => Color::Blue,
    CoreColor::Magenta => Color::Magenta,
    CoreColor::Cyan => Color::Cyan,
    CoreColor::Gray => Color::Gray,
    CoreColor::DarkGray => Color::DarkGray,
    CoreColor::LightRed => Color::LightRed,
    CoreColor::LightGreen => Color::LightGreen,
    CoreColor::LightYellow => Color::LightYellow,
    CoreColor::LightBlue => Color::LightBlue,
    CoreColor::LightMagenta => Color::LightMagenta,
    CoreColor::LightCyan => Color::LightCyan,
    CoreColor::White => Color::White,
    CoreColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
    CoreColor::Indexed(i) => Color::Indexed(i),
  }
}

fn parse_lines(output: &[u8]) -> Vec<String> {
  String::from_utf8_lossy(output)
    .lines()
    .map(|line| line.trim())
    .filter(|line| !line.is_empty())
    .map(|line| line.to_string())
    .collect()
}

fn is_symlink(path: &Path) -> Result<bool> {
  match fs::symlink_metadata(path) {
    Ok(meta) => Ok(meta.file_type().is_symlink()),
    Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
    Err(err) => Err(err.into()),
  }
}

fn command_exists(cmd: &str) -> bool {
  which::which(cmd).is_ok()
}

fn term_contains(value: &str) -> bool {
  std::env::var("TERM")
    .unwrap_or_default()
    .to_lowercase()
    .contains(value)
}

trait ItemView {
  fn label(&self) -> String;
  fn preview_path(&self) -> Option<&PathBuf>;
}

impl ItemView for OptionItem {
  fn label(&self) -> String {
    self.label.clone()
  }

  fn preview_path(&self) -> Option<&PathBuf> {
    self.preview.as_ref()
  }
}

impl ItemView for LabeledItem {
  fn label(&self) -> String {
    self.label.clone()
  }

  fn preview_path(&self) -> Option<&PathBuf> {
    self.preview.as_ref()
  }
}
