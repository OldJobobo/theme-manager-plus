use anyhow::{anyhow, Result};
use crossterm::event::{
  self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
  KeyboardEnhancementFlags, MouseButton, MouseEventKind, PopKeyboardEnhancementFlags,
  PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::fs;
use std::io::{stdout, Stdout};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use ansi_to_tui::IntoText;
use ratatui_core::layout::Alignment as CoreAlignment;
use ratatui_core::style::{Color as CoreColor, Modifier as CoreModifier, Style as CoreStyle};
use ratatui_core::text::{Line as CoreLine, Span as CoreSpan, Text as CoreText};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use tempfile::TempDir;

use crate::config::ResolvedConfig;
use crate::paths::title_case_theme;
use crate::preview;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusArea {
  List,
  Code,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowseTab {
  Theme,
  Waybar,
  Starship,
  Review,
}

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

struct PickerState {
  list_state: ListState,
  last_code_index: Option<usize>,
  last_code: Text<'static>,
  last_preview_index: Option<usize>,
  last_preview: Option<PathBuf>,
  preview_dirty: bool,
  last_preview_text: Text<'static>,
  code_scroll: u16,
  focus: FocusArea,
  image_visible: bool,
  force_clear: bool,
}

impl PickerState {
  fn new() -> Self {
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    Self {
      list_state,
      last_code_index: None,
      last_code: Text::from("Loading preview..."),
      last_preview_index: None,
      last_preview: None,
      preview_dirty: false,
      last_preview_text: Text::from(""),
      code_scroll: 0,
      focus: FocusArea::List,
      image_visible: false,
      force_clear: false,
    }
  }
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
      PreviewBackendKind::Kitty => {
        if path.is_some() {
          Text::from("")
        } else {
          Text::from("No preview available.")
        }
      }
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
  let mut terminal = setup_terminal()?;
  let mut tab = BrowseTab::Theme;
  let tab_titles = ["Theme", "Waybar", "Starship", "Review"];
  let mut tab_ranges: Vec<(u16, u16, usize)> = Vec::new();
  let mut active_list_inner = Rect::ZERO;
  let mut active_code_inner = Rect::ZERO;
  let mut active_code_area = Rect::ZERO;
  let mut tab_area = Rect::ZERO;
  let mut last_repeat_key: Option<(KeyCode, KeyModifiers)> = None;
  let mut last_repeat_at = Instant::now();
  let mut last_press_key: Option<(KeyCode, KeyModifiers, Instant)> = None;
  let mut status_message = String::new();
  let mut status_tab = BrowseTab::Theme;
  let mut status_at = Instant::now();

  let mut theme_state = PickerState::new();
  ensure_selected(&mut theme_state.list_state, theme_items.len());
  let mut selected_theme = current_theme_value(&theme_items, &theme_state.list_state)?;
  let mut theme_path = config.theme_root_dir.join(&selected_theme);

  let mut waybar_items = build_waybar_items(config, &theme_path)?;
  let mut starship_items = build_starship_items(config, &theme_path)?;
  let mut waybar_state = PickerState::new();
  let mut starship_state = PickerState::new();
  ensure_selected(&mut waybar_state.list_state, waybar_items.len());
  ensure_selected(&mut starship_state.list_state, starship_items.len());

  loop {
    terminal.draw(|frame| {
      let size = frame.area();
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);
      tab_area = chunks[0];
      let content_area = chunks[1];

      render_tab_bar(frame, tab_area, &tab_titles, tab, &mut tab_ranges);
      let status_active = !status_message.is_empty()
        && status_at.elapsed() < Duration::from_millis(1200);

      match tab {
        BrowseTab::Theme => {
          let areas = render_picker(
            frame,
            content_area,
            "Select theme",
            "Image Preview",
            &theme_items,
            &mut theme_state,
            &backend,
            |idx| {
              let theme_path = config.theme_root_dir.join(&theme_items[idx].value);
              load_code_preview(
                "hyprland.conf",
                theme_path.join("hyprland.conf"),
                "conf",
              )
            },
            |idx| theme_items[idx].preview.clone(),
            |_idx| None,
            true,
            if status_active && status_tab == BrowseTab::Theme {
              Some(status_message.as_str())
            } else {
              None
            },
          );
          active_list_inner = areas.list_inner;
          active_code_inner = areas.code_inner;
          active_code_area = areas.code_area;
        }
        BrowseTab::Waybar => {
          let areas = render_picker(
            frame,
            content_area,
            "Select Waybar",
            "Image Preview",
            &waybar_items,
            &mut waybar_state,
            &backend,
            |idx| build_waybar_code_preview(config, &theme_path, &waybar_items[idx]),
            |idx| waybar_items[idx].preview.clone(),
            |_idx| None,
            true,
            if status_active && status_tab == BrowseTab::Waybar {
              Some(status_message.as_str())
            } else {
              None
            },
          );
          active_list_inner = areas.list_inner;
          active_code_inner = areas.code_inner;
          active_code_area = areas.code_area;
        }
        BrowseTab::Starship => {
          let areas = render_picker(
            frame,
            content_area,
            "Select Starship",
            "Prompt Preview",
            &starship_items,
            &mut starship_state,
            &backend,
            |idx| build_starship_code_preview(config, &theme_path, &starship_items[idx]),
            |_idx| None,
            |idx| Some(build_starship_prompt_preview(config, &theme_path, &starship_items[idx])),
            false,
            if status_active && status_tab == BrowseTab::Starship {
              Some(status_message.as_str())
            } else {
              None
            },
          );
          active_list_inner = areas.list_inner;
          active_code_inner = areas.code_inner;
          active_code_area = areas.code_area;
        }
        BrowseTab::Review => {
          active_list_inner = Rect::ZERO;
          active_code_inner = Rect::ZERO;
          active_code_area = Rect::ZERO;
          render_review(
            frame,
            content_area,
            &selected_theme,
            current_waybar_label(&waybar_items, &waybar_state.list_state),
            current_starship_label(&starship_items, &starship_state.list_state),
          );
        }
      }
    })?;

    if event::poll(Duration::from_millis(200))? {
      match event::read()? {
        Event::Key(key) => {
          if key.kind == KeyEventKind::Release {
            continue;
          }
          let now = Instant::now();
          let is_repeat = key.kind == event::KeyEventKind::Repeat;
          if is_repeat {
            if let Some((last_code, last_mod, last_at)) = last_press_key {
              if last_code == key.code && last_mod == key.modifiers {
                if now.duration_since(last_at) < Duration::from_millis(150) {
                  continue;
                }
              }
            }
            if let Some((last_code, last_mod)) = last_repeat_key {
              if last_code == key.code && last_mod == key.modifiers {
                if now.duration_since(last_repeat_at) < Duration::from_millis(35) {
                  continue;
                }
              }
            }
            last_repeat_key = Some((key.code, key.modifiers));
            last_repeat_at = now;
          } else {
            last_press_key = Some((key.code, key.modifiers, now));
          }
          if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
            cleanup_terminal(&mut terminal)?;
            return Ok(None);
          }
          if key.code == KeyCode::Tab {
            tab = previous_tab(tab);
            clear_kitty_preview(&backend);
            mark_force_clear(&mut theme_state, &mut waybar_state, &mut starship_state);
            continue;
          }
          if key.code == KeyCode::BackTab {
            tab = next_tab(tab);
            clear_kitty_preview(&backend);
            mark_force_clear(&mut theme_state, &mut waybar_state, &mut starship_state);
            continue;
          }
          if tab == BrowseTab::Review
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Enter | KeyCode::Char('m') | KeyCode::Char('j'))
          {
            let selection = BrowseSelection {
              theme: selected_theme.clone(),
              waybar: current_waybar_selection(&waybar_items, &waybar_state.list_state),
              starship: current_starship_selection(
                &starship_items,
                &starship_state.list_state,
                &theme_path,
              ),
            };
            cleanup_terminal(&mut terminal)?;
            return Ok(Some(selection));
          }
          if key.code == KeyCode::Enter && tab != BrowseTab::Review {
            status_tab = tab;
            status_at = Instant::now();
            status_message = match tab {
              BrowseTab::Theme => "Theme selected".to_string(),
              BrowseTab::Waybar => "Waybar selected".to_string(),
              BrowseTab::Starship => "Starship selected".to_string(),
              BrowseTab::Review => String::new(),
            };
            tab = next_tab(tab);
            continue;
          }

          if let Some(state) = active_picker_mut(tab, &mut theme_state, &mut waybar_state, &mut starship_state) {
            let items_len = active_items_len(tab, &theme_items, &waybar_items, &starship_items);
            match key.code {
              KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('h') => match state.focus {
                FocusArea::List => {
                  let new_index = previous_index(state.list_state.selected(), items_len);
                  state.list_state.select(Some(new_index));
                }
                FocusArea::Code => {
                  state.code_scroll = state.code_scroll.saturating_sub(1);
                }
              },
              KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('l') => match state.focus {
                FocusArea::List => {
                  let new_index = next_index(state.list_state.selected(), items_len);
                  state.list_state.select(Some(new_index));
                }
                FocusArea::Code => {
                  state.code_scroll = state.code_scroll.saturating_add(1);
                }
              },
              KeyCode::PageUp => {
                let step = inner_rect(active_code_area).height.max(1);
                state.code_scroll = state.code_scroll.saturating_sub(step);
              }
              KeyCode::PageDown => {
                let step = inner_rect(active_code_area).height.max(1);
                state.code_scroll = state.code_scroll.saturating_add(step);
              }
              KeyCode::Home => match state.focus {
                FocusArea::List => state.list_state.select(Some(0)),
                FocusArea::Code => state.code_scroll = 0,
              },
              KeyCode::End => match state.focus {
                FocusArea::List => {
                  state
                    .list_state
                    .select(Some(items_len.saturating_sub(1)));
                }
                FocusArea::Code => {
                  let code_height = inner_rect(active_code_area).height as usize;
                  let max_scroll = state
                    .last_code
                    .lines
                    .len()
                    .saturating_sub(code_height.max(1));
                  state.code_scroll = max_scroll as u16;
                }
              },
              _ => {}
            }
          }
        }
        Event::Mouse(mouse) => match mouse.kind {
          MouseEventKind::Down(MouseButton::Left) => {
            if tab_area.contains(Position {
              x: mouse.column,
              y: mouse.row,
            }) {
              if let Some(index) = tab_index_from_click(&tab_ranges, mouse.column) {
                tab = tab_from_index(index);
                clear_kitty_preview(&backend);
                mark_force_clear(&mut theme_state, &mut waybar_state, &mut starship_state);
                continue;
              }
            }

            if let Some(state) =
              active_picker_mut(tab, &mut theme_state, &mut waybar_state, &mut starship_state)
            {
              let position = Position {
                x: mouse.column,
                y: mouse.row,
              };
              if active_list_inner.contains(position) {
                state.focus = FocusArea::List;
                let items_len = active_items_len(tab, &theme_items, &waybar_items, &starship_items);
                select_index_at_row(
                  &mut state.list_state,
                  active_list_inner,
                  mouse.row,
                  items_len,
                );
              } else if active_code_inner.contains(position) {
                state.focus = FocusArea::Code;
              }
            }
          }
          MouseEventKind::ScrollUp => {
            if let Some(state) =
              active_picker_mut(tab, &mut theme_state, &mut waybar_state, &mut starship_state)
            {
              let position = Position {
                x: mouse.column,
                y: mouse.row,
              };
              if active_list_inner.contains(position) {
                state.focus = FocusArea::List;
                let items_len = active_items_len(tab, &theme_items, &waybar_items, &starship_items);
                let new_index = previous_index(state.list_state.selected(), items_len);
                state.list_state.select(Some(new_index));
              } else if active_code_inner.contains(position) {
                state.focus = FocusArea::Code;
                state.code_scroll = state.code_scroll.saturating_sub(1);
              }
            }
          }
          MouseEventKind::ScrollDown => {
            if let Some(state) =
              active_picker_mut(tab, &mut theme_state, &mut waybar_state, &mut starship_state)
            {
              let position = Position {
                x: mouse.column,
                y: mouse.row,
              };
              if active_list_inner.contains(position) {
                state.focus = FocusArea::List;
                let items_len = active_items_len(tab, &theme_items, &waybar_items, &starship_items);
                let new_index = next_index(state.list_state.selected(), items_len);
                state.list_state.select(Some(new_index));
              } else if active_code_inner.contains(position) {
                state.focus = FocusArea::Code;
                state.code_scroll = state.code_scroll.saturating_add(1);
              }
            }
          }
          _ => {}
        },
        _ => {}
      }
    }

    let new_theme = current_theme_value(&theme_items, &theme_state.list_state)?;
    if new_theme != selected_theme {
      selected_theme = new_theme;
      theme_path = config.theme_root_dir.join(&selected_theme);
      let waybar_key = selected_item_key(&waybar_items, &waybar_state.list_state);
      let starship_key = selected_item_key(&starship_items, &starship_state.list_state);

      waybar_items = build_waybar_items(config, &theme_path)?;
      starship_items = build_starship_items(config, &theme_path)?;

      reset_picker_cache(&mut waybar_state);
      reset_picker_cache(&mut starship_state);

      select_item_by_key(&mut waybar_state.list_state, &waybar_items, waybar_key);
      select_item_by_key(&mut starship_state.list_state, &starship_items, starship_key);
      ensure_selected(&mut waybar_state.list_state, waybar_items.len());
      ensure_selected(&mut starship_state.list_state, starship_items.len());
    }
  }
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

fn build_waybar_items(config: &ResolvedConfig, theme_path: &Path) -> Result<Vec<LabeledItem>> {
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

  Ok(items)
}

fn build_starship_items(config: &ResolvedConfig, theme_path: &Path) -> Result<Vec<LabeledItem>> {
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

  Ok(items)
}

fn build_waybar_code_preview(
  config: &ResolvedConfig,
  theme_path: &Path,
  item: &LabeledItem,
) -> Text<'static> {
  match item.kind.as_str() {
    "default" => Text::from("Using Omarchy default Waybar config."),
    "theme" => {
      let base = theme_path.join("waybar-theme");
      let parts = vec![
        ("config.jsonc", base.join("config.jsonc"), "json"),
        ("style.css", base.join("style.css"), "css"),
      ];
      load_multi_code_preview(&parts)
    }
    _ => {
      let base = config.waybar_themes_dir.join(&item.value);
      let parts = vec![
        ("config.jsonc", base.join("config.jsonc"), "json"),
        ("style.css", base.join("style.css"), "css"),
      ];
      load_multi_code_preview(&parts)
    }
  }
}

fn build_starship_code_preview(
  config: &ResolvedConfig,
  theme_path: &Path,
  item: &LabeledItem,
) -> Text<'static> {
  match item.kind.as_str() {
    "default" => Text::from("No Starship config change."),
    "theme" => load_code_preview(
      "starship.yaml",
      theme_path.join("starship.yaml"),
      "yaml",
    ),
    "preset" => {
      let preset = item.value.as_str();
      let output = Command::new("starship")
        .args(["preset", preset])
        .output();
      let output = match output {
        Ok(output) if output.status.success() => output.stdout,
        _ => return Text::from(format!("Failed to load preset: {preset}")),
      };
      load_code_preview_from_string("preset.toml", &String::from_utf8_lossy(&output), "toml")
    }
    _ => load_code_preview(
      &format!("{}.toml", item.value),
      config
        .starship_themes_dir
        .join(format!("{}.toml", item.value)),
      "toml",
    ),
  }
}

fn build_starship_prompt_preview(
  config: &ResolvedConfig,
  theme_path: &Path,
  item: &LabeledItem,
) -> Text<'static> {
  render_starship_prompt_preview(config, theme_path, item)
}

fn load_multi_code_preview(parts: &[(&str, PathBuf, &str)]) -> Text<'static> {
  let mut combined = Text::from("");
  let mut first = true;
  for (title, path, syntax) in parts {
    if !first {
      combined.lines.push(Line::from(""));
    }
    first = false;
    let mut header = Text::from(vec![
      Line::from(format!("=== {} ===", title)),
      Line::from(""),
    ]);
    combined.lines.append(&mut header.lines);
    let block = load_code_preview(title, path.clone(), syntax);
    combined.lines.extend(block.lines);
  }
  combined
}

fn load_code_preview(title: &str, path: PathBuf, syntax: &str) -> Text<'static> {
  if !path.is_file() {
    return Text::from(format!("Missing {} at {}", title, path.to_string_lossy()));
  }
  match fs::read_to_string(&path) {
    Ok(content) => load_code_preview_from_string(title, &content, syntax),
    Err(_) => Text::from(format!("Failed to read {}", title)),
  }
}

fn load_code_preview_from_string(title: &str, content: &str, syntax: &str) -> Text<'static> {
  let mut lines = Vec::new();
  lines.push(Line::from(format!("=== {} ===", title)));
  lines.push(Line::from(""));
  let highlighted = highlight_code(content, syntax);
  lines.extend(highlighted.lines);
  Text::from(lines)
}

fn highlight_code(content: &str, syntax: &str) -> Text<'static> {
  let ps = SyntaxSet::load_defaults_newlines();
  let ts = ThemeSet::load_defaults();
  let theme = ts
    .themes
    .get("base16-ocean.dark")
    .or_else(|| ts.themes.values().next())
    .expect("theme");
  let syntax_ref = ps
    .find_syntax_by_extension(syntax)
    .unwrap_or_else(|| ps.find_syntax_plain_text());
  let mut h = HighlightLines::new(syntax_ref, theme);
  let mut out = String::new();
  for line in content.lines() {
    let ranges = h.highlight_line(line, &ps).unwrap_or_default();
    out.push_str(&as_24_bit_terminal_escaped(&ranges[..], false));
    out.push('\n');
  }
  match out.as_bytes().into_text() {
    Ok(text) => convert_text(text),
    Err(_) => Text::from(content.to_string()),
  }
}

fn render_starship_prompt_preview(
  config: &ResolvedConfig,
  theme_path: &Path,
  item: &LabeledItem,
) -> Text<'static> {
  if item.kind.as_str() == "default" {
    return Text::from("No Starship config change.\n\nThe current Omarchy theme prompt will be used.");
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

  let config_path = match item.kind.as_str() {
    "theme" => {
      let path = theme_path.join("starship.yaml");
      if !path.is_file() {
        return Text::from("Theme-specific Starship config not found.");
      }
      path
    }
    "preset" => {
      let preset_name = item.value.as_str();
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
      let path = config
        .starship_themes_dir
        .join(format!("{}.toml", item.value));
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

  let width = 100u16;
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

struct PickerAreas {
  list_inner: Rect,
  code_inner: Rect,
  code_area: Rect,
}

fn render_picker<T: ItemView>(
  frame: &mut Frame,
  area: Rect,
  title: &str,
  preview_title: &str,
  items: &[T],
  state: &mut PickerState,
  backend: &PreviewBackend,
  code_preview: impl Fn(usize) -> Text<'static>,
  image_preview: impl Fn(usize) -> Option<PathBuf>,
  preview_text: impl Fn(usize) -> Option<Text<'static>>,
  tall_image_preview: bool,
  status: Option<&str>,
) -> PickerAreas {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
      if tall_image_preview {
        [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref()
      } else {
        [Constraint::Percentage(65), Constraint::Percentage(35)].as_ref()
      },
    )
    .split(area);
  let top_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
    .split(chunks[0]);
  let image_area = inner_rect(chunks[1]);
  let list_area = top_chunks[0];
  let list_inner = list_inner_rect(list_area);
  let code_area = top_chunks[1];
  let code_inner = inner_rect(code_area);

  let list_items: Vec<ListItem> = items
    .iter()
    .map(|item| ListItem::new(Line::from(item.label())))
    .collect();
  let list_title = if let Some(status) = status {
    format!("{title}  [{status}]")
  } else {
    title.to_string()
  };
  let list_block = Block::default()
    .title(list_title)
    .borders(Borders::ALL)
    .border_style(if state.focus == FocusArea::List {
      Style::default()
        .fg(if status.is_some() {
          Color::Green
        } else {
          Color::Yellow
        })
    } else {
      Style::default()
    });
  let list = List::new(list_items)
    .block(list_block)
    .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    .highlight_symbol(">> ");
  frame.render_stateful_widget(list, list_area, &mut state.list_state);

  let selected = selected_index(&state.list_state, items.len());
  let preview_path = items
    .get(selected)
    .and_then(|_item| image_preview(selected));

  if Some(selected) != state.last_code_index {
    state.last_code = code_preview(selected);
    state.code_scroll = 0;
    state.last_code_index = Some(selected);
  }

  let max_scroll = state
    .last_code
    .lines
    .len()
    .saturating_sub(code_inner.height.max(1) as usize);
  if state.code_scroll as usize > max_scroll {
    state.code_scroll = max_scroll as u16;
  }
  let code_block = Block::default()
    .title("Code Preview")
    .borders(Borders::ALL)
    .border_style(if state.focus == FocusArea::Code {
      Style::default().fg(Color::Yellow)
    } else {
      Style::default()
    });
  let code = Paragraph::new(state.last_code.clone())
    .block(code_block)
    .scroll((state.code_scroll, 0))
    .wrap(Wrap { trim: false });
  frame.render_widget(code, code_area);

  if Some(selected) != state.last_preview_index {
    if let Some(text) = preview_text(selected) {
      state.last_preview_text = text;
      if state.last_preview.is_some() {
        state.preview_dirty = true;
      }
      state.last_preview = None;
    } else {
      state.last_preview_text = Text::from("");
      if state.last_preview != preview_path {
        state.last_preview = preview_path.clone();
        state.preview_dirty = true;
      }
    }
    state.last_preview_index = Some(selected);
  }

  if state.force_clear {
    backend.render(None, image_area);
    state.image_visible = false;
    state.force_clear = false;
  }

  let wants_image = state.last_preview_text.lines.is_empty() && state.last_preview.is_some();
  let should_render = state.preview_dirty || (wants_image && !state.image_visible);
  if should_render {
    backend.render(state.last_preview.as_deref(), image_area);
    state.image_visible = wants_image;
    state.preview_dirty = false;
  } else if !wants_image && state.image_visible {
    backend.render(None, image_area);
    state.image_visible = false;
  }

  let preview_text_rendered = if state.last_preview_text.lines.is_empty() {
    backend.text_preview(state.last_preview.as_deref(), image_area)
  } else {
    state.last_preview_text.clone()
  };
  let preview = Paragraph::new(preview_text_rendered)
    .block(Block::default().title(preview_title).borders(Borders::ALL));
  frame.render_widget(preview, chunks[1]);

  PickerAreas {
    list_inner,
    code_inner,
    code_area,
  }
}

fn render_review(
  frame: &mut Frame,
  area: Rect,
  selected_theme: &str,
  waybar_label: String,
  starship_label: String,
) {
  let lines = vec![
    Line::from("=== Review Selections ==="),
    Line::from(""),
    Line::from(format!("Theme: {}", title_case_theme(selected_theme))),
    Line::from(format!("Waybar: {}", waybar_label)),
    Line::from(format!("Starship: {}", starship_label)),
    Line::from(""),
    Line::from("Apply: Ctrl+Enter"),
    Line::from("Cancel: Esc"),
    Line::from("Switch tabs: Tab / Shift+Tab (or click tab bar)"),
  ];
  let review = Paragraph::new(Text::from(lines))
    .block(Block::default().title("Review").borders(Borders::ALL))
    .wrap(Wrap { trim: false });
  frame.render_widget(review, area);
}

fn render_tab_bar(
  frame: &mut Frame,
  area: Rect,
  titles: &[&str],
  active: BrowseTab,
  ranges: &mut Vec<(u16, u16, usize)>,
) {
  ranges.clear();
  let block = Block::default().borders(Borders::ALL);
  let inner = block.inner(area);
  frame.render_widget(block, area);

  let mut spans = Vec::new();
  let mut cursor = inner.x;
  let active_index = tab_index(active);

  for (idx, title) in titles.iter().enumerate() {
    let label = format!(" {} ", title);
    let width = label.len() as u16;
    let style = if idx == active_index {
      Style::default()
        .fg(Color::Black)
        .bg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::default()
    };
    spans.push(Span::styled(label, style));
    ranges.push((cursor, cursor + width.saturating_sub(1), idx));
    cursor = cursor.saturating_add(width);
    if idx + 1 < titles.len() {
      spans.push(Span::raw("│"));
      cursor = cursor.saturating_add(1);
    }
  }

  let line = Line::from(spans);
  let tabs = Paragraph::new(line).alignment(ratatui::layout::Alignment::Left);
  frame.render_widget(tabs, inner);
}

fn clear_kitty_preview(backend: &PreviewBackend) {
  if matches!(backend.kind, PreviewBackendKind::Kitty) {
    let _ = Command::new("kitty")
      .args(["+kitten", "icat", "--clear", "--stdin=no"])
      .status();
  }
}

fn mark_force_clear(
  theme: &mut PickerState,
  waybar: &mut PickerState,
  starship: &mut PickerState,
) {
  theme.force_clear = true;
  waybar.force_clear = true;
  starship.force_clear = true;
}

fn active_picker_mut<'a>(
  tab: BrowseTab,
  theme: &'a mut PickerState,
  waybar: &'a mut PickerState,
  starship: &'a mut PickerState,
) -> Option<&'a mut PickerState> {
  match tab {
    BrowseTab::Theme => Some(theme),
    BrowseTab::Waybar => Some(waybar),
    BrowseTab::Starship => Some(starship),
    BrowseTab::Review => None,
  }
}

fn active_items_len(
  tab: BrowseTab,
  theme: &[OptionItem],
  waybar: &[LabeledItem],
  starship: &[LabeledItem],
) -> usize {
  match tab {
    BrowseTab::Theme => theme.len(),
    BrowseTab::Waybar => waybar.len(),
    BrowseTab::Starship => starship.len(),
    BrowseTab::Review => 0,
  }
}

fn tab_index(tab: BrowseTab) -> usize {
  match tab {
    BrowseTab::Theme => 0,
    BrowseTab::Waybar => 1,
    BrowseTab::Starship => 2,
    BrowseTab::Review => 3,
  }
}

fn tab_from_index(index: usize) -> BrowseTab {
  match index {
    0 => BrowseTab::Theme,
    1 => BrowseTab::Waybar,
    2 => BrowseTab::Starship,
    _ => BrowseTab::Review,
  }
}

fn next_tab(tab: BrowseTab) -> BrowseTab {
  tab_from_index((tab_index(tab) + 1) % 4)
}

fn previous_tab(tab: BrowseTab) -> BrowseTab {
  tab_from_index((tab_index(tab) + 3) % 4)
}

fn tab_index_from_click(ranges: &[(u16, u16, usize)], column: u16) -> Option<usize> {
  ranges
    .iter()
    .find(|(start, end, _)| column >= *start && column <= *end)
    .map(|(_, _, idx)| *idx)
}

fn current_theme_value(items: &[OptionItem], state: &ListState) -> Result<String> {
  if items.is_empty() {
    return Err(anyhow!("no themes available"));
  }
  let index = selected_index(state, items.len());
  Ok(items[index].value.clone())
}

fn current_waybar_label(items: &[LabeledItem], state: &ListState) -> String {
  if items.is_empty() {
    return "No options".to_string();
  }
  if items.len() == 1 && items[0].kind == "default" {
    return "Use defaults".to_string();
  }
  let index = selected_index(state, items.len());
  let item = &items[index];
  match item.kind.as_str() {
    "default" => "Omarchy default".to_string(),
    "theme" => "Theme waybar".to_string(),
    _ => item.label.clone(),
  }
}

fn current_starship_label(items: &[LabeledItem], state: &ListState) -> String {
  if items.is_empty() {
    return "No options".to_string();
  }
  if items.len() == 1 && items[0].kind == "default" {
    return "Use defaults".to_string();
  }
  let index = selected_index(state, items.len());
  let item = &items[index];
  match item.kind.as_str() {
    "default" => "Omarchy default".to_string(),
    "theme" => "Theme starship".to_string(),
    _ => item.label.clone(),
  }
}

fn current_waybar_selection(items: &[LabeledItem], state: &ListState) -> WaybarSelection {
  if items.is_empty() {
    return WaybarSelection::UseDefaults;
  }
  if items.len() == 1 && items[0].kind == "default" {
    return WaybarSelection::UseDefaults;
  }
  let index = selected_index(state, items.len());
  match items[index].kind.as_str() {
    "default" => WaybarSelection::None,
    "theme" => WaybarSelection::Auto,
    _ => WaybarSelection::Named(items[index].value.clone()),
  }
}

fn current_starship_selection(
  items: &[LabeledItem],
  state: &ListState,
  theme_path: &Path,
) -> StarshipSelection {
  if items.is_empty() {
    return StarshipSelection::UseDefaults;
  }
  if items.len() == 1 && items[0].kind == "default" {
    return StarshipSelection::UseDefaults;
  }
  let index = selected_index(state, items.len());
  match items[index].kind.as_str() {
    "default" => StarshipSelection::None,
    "theme" => StarshipSelection::Theme(theme_path.join("starship.yaml")),
    "preset" => StarshipSelection::Preset(items[index].value.clone()),
    _ => StarshipSelection::Named(items[index].value.clone()),
  }
}

fn selected_item_key(items: &[LabeledItem], state: &ListState) -> Option<(String, String)> {
  if items.is_empty() {
    return None;
  }
  let index = selected_index(state, items.len());
  Some((items[index].kind.clone(), items[index].value.clone()))
}

fn select_item_by_key(
  state: &mut ListState,
  items: &[LabeledItem],
  key: Option<(String, String)>,
) {
  if let Some((kind, value)) = key {
    if let Some(index) = items
      .iter()
      .position(|item| item.kind == kind && item.value == value)
    {
      state.select(Some(index));
    }
  }
}

fn reset_picker_cache(state: &mut PickerState) {
  state.last_code_index = None;
  state.last_preview_index = None;
  state.last_preview = None;
  state.preview_dirty = false;
  state.last_preview_text = Text::from("");
  state.code_scroll = 0;
  state.image_visible = false;
  state.force_clear = true;
}

fn ensure_selected(state: &mut ListState, len: usize) {
  if len == 0 {
    state.select(None);
    return;
  }
  let selected = state.selected().unwrap_or(0);
  let clamped = selected.min(len.saturating_sub(1));
  state.select(Some(clamped));
}

fn selected_index(state: &ListState, len: usize) -> usize {
  if len == 0 {
    return 0;
  }
  state.selected().unwrap_or(0).min(len.saturating_sub(1))
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
  enable_raw_mode()?;
  let mut stdout = stdout();
  execute!(stdout, terminal::EnterAlternateScreen, EnableMouseCapture)?;
  let _ = execute!(
    stdout,
    PushKeyboardEnhancementFlags(
      KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
    )
  );
  let backend = CrosstermBackend::new(stdout);
  Terminal::new(backend).map_err(|err| anyhow!("failed to init terminal: {err}"))
}

fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    DisableMouseCapture,
    PopKeyboardEnhancementFlags,
    terminal::LeaveAlternateScreen
  )?;
  terminal.show_cursor()?;
  Ok(())
}

fn inner_rect(rect: Rect) -> Rect {
  let pad = 2;
  Rect {
    x: rect.x.saturating_add(pad),
    y: rect.y.saturating_add(pad),
    width: rect.width.saturating_sub(pad * 2),
    height: rect.height.saturating_sub(pad * 2),
  }
}

fn list_inner_rect(rect: Rect) -> Rect {
  let pad = 1;
  Rect {
    x: rect.x.saturating_add(pad),
    y: rect.y.saturating_add(pad),
    width: rect.width.saturating_sub(pad * 2),
    height: rect.height.saturating_sub(pad * 2),
  }
}

fn select_index_at_row(state: &mut ListState, rect: Rect, row: u16, len: usize) {
  if len == 0 || rect.height == 0 {
    return;
  }
  let offset = state.offset();
  let relative = row.saturating_sub(rect.y) as usize;
  let index = offset.saturating_add(relative);
  if index < len {
    state.select(Some(index));
  }
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
}

impl ItemView for OptionItem {
  fn label(&self) -> String {
    self.label.clone()
  }
}

impl ItemView for LabeledItem {
  fn label(&self) -> String {
    self.label.clone()
  }
}
