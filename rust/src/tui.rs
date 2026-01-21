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
use crate::paths::{normalize_theme_name, title_case_theme};
use crate::theme_ops::{starship_from_defaults, waybar_from_defaults, StarshipMode, WaybarMode};
use crate::theme_ops;
use crate::presets;
use crate::preview;

const APP_TITLE: &str = concat!("Theme Manager+ v", env!("CARGO_PKG_VERSION"));

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
  Presets,
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
  search_query: String,
  last_query: String,
  filtered_indices: Vec<usize>,
  last_selected: Option<usize>,
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
      search_query: String::new(),
      last_query: String::new(),
      filtered_indices: Vec::new(),
      last_selected: None,
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
  let themes = theme_ops::list_theme_entries_for_config(config)?;
  if themes.is_empty() {
    return Err(anyhow!("no themes available"));
  }

  let theme_items: Vec<OptionItem> = themes
    .into_iter()
    .map(|name| {
      let label = title_case_theme(&name);
      let theme_path = theme_ops::resolve_theme_path(config, &name)?;
      let preview_path = preview::find_theme_preview(&theme_path);
      Ok(OptionItem {
        label,
        value: name,
        preview: preview_path,
      })
    })
    .collect::<Result<Vec<_>>>()?;

  let backend = PreviewBackend::detect();
  let mut terminal = setup_terminal()?;
  let mut tab = BrowseTab::Theme;
  let tab_titles = ["Theme", "Waybar", "Starship", "Review", "Presets"];
  let mut tab_ranges: Vec<(u16, u16, usize)> = Vec::new();
  let mut active_search_area = Rect::ZERO;
  let mut active_list_inner = Rect::ZERO;
  let mut active_code_inner = Rect::ZERO;
  let mut active_code_area = Rect::ZERO;
  let mut tab_area = Rect::ZERO;
  let mut status_area = Rect::ZERO;
  let mut last_repeat_key: Option<(KeyCode, KeyModifiers)> = None;
  let mut last_repeat_at = Instant::now();
  let mut last_press_key: Option<(KeyCode, KeyModifiers, Instant)> = None;
  let mut status_message = String::new();
  let mut status_tab = BrowseTab::Theme;
  let mut status_at = Instant::now();
  let mut preset_save_active = false;
  let mut preset_save_input = String::new();

  let mut theme_state = PickerState::new();
  rebuild_filtered(&mut theme_state, &theme_items);
  let mut selected_theme = current_theme_value(&theme_items, &theme_state)
    .ok_or_else(|| anyhow!("no themes available"))?;
  let mut theme_path = theme_ops::resolve_theme_path(config, &selected_theme)?;

  let mut waybar_items = build_waybar_items(config, &theme_path)?;
  let mut starship_items = build_starship_items(config, &theme_path)?;
  let mut waybar_state = PickerState::new();
  let mut starship_state = PickerState::new();
  rebuild_filtered(&mut waybar_state, &waybar_items);
  rebuild_filtered(&mut starship_state, &starship_items);

  let mut preset_file = presets::load_presets()?;
  let mut preset_items = build_preset_items(&preset_file);
  let mut preset_state = PickerState::new();
  rebuild_filtered(&mut preset_state, &preset_items);

  loop {
    terminal.draw(|frame| {
      let size = frame.area();
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
          Constraint::Length(2),
          Constraint::Min(0),
          Constraint::Length(1),
        ])
        .split(size);
      tab_area = chunks[0];
      let content_area = chunks[1];
      status_area = chunks[2];

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
              let theme_path = theme_ops::resolve_theme_path(config, &theme_items[idx].value)?;
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
          active_search_area = areas.search_area;
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
          active_search_area = areas.search_area;
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
          active_search_area = areas.search_area;
          active_list_inner = areas.list_inner;
          active_code_inner = areas.code_inner;
          active_code_area = areas.code_area;
        }
        BrowseTab::Presets => {
          let areas = render_preset_picker(
            frame,
            content_area,
            &preset_items,
            &mut preset_state,
            |idx| preset_summary_text(config, &preset_file, &preset_items[idx]),
            if status_active && status_tab == BrowseTab::Presets {
              Some(status_message.as_str())
            } else {
              None
            },
          );
          active_search_area = areas.search_area;
          active_list_inner = areas.list_inner;
          active_code_inner = areas.code_inner;
          active_code_area = areas.code_area;
        }
        BrowseTab::Review => {
          active_search_area = Rect::ZERO;
          active_list_inner = Rect::ZERO;
          active_code_inner = Rect::ZERO;
          active_code_area = Rect::ZERO;
          render_review(
            frame,
            content_area,
            &selected_theme,
            current_waybar_label(&waybar_items, &waybar_state),
            current_starship_label(&starship_items, &starship_state),
          );
        }
      }

      render_status_bar(
        frame,
        status_area,
        tab,
        &selected_theme,
        current_waybar_label(&waybar_items, &waybar_state),
        current_starship_label(&starship_items, &starship_state),
        status_active.then_some(status_message.as_str()),
        preset_save_active,
        &preset_save_input,
      );
    })?;

    if event::poll(Duration::from_millis(200))? {
      let mut handled_nav = false;
      'event_loop: loop {
        let next_event = event::read()?;
        match next_event {
          Event::Key(key) => {
            if key.kind == KeyEventKind::Release {
              if !event::poll(Duration::from_millis(0))? {
                break 'event_loop;
              }
              continue 'event_loop;
            }
            let is_nav_key = matches!(
              key.code,
              KeyCode::Up
                | KeyCode::Down
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End
            );
            if is_nav_key {
              if handled_nav {
                if !event::poll(Duration::from_millis(0))? {
                  break 'event_loop;
                }
                continue 'event_loop;
              }
              handled_nav = true;
            }
            let now = Instant::now();
            if preset_save_active {
              if key.kind == KeyEventKind::Repeat {
                if !event::poll(Duration::from_millis(0))? {
                  break 'event_loop;
                }
                continue 'event_loop;
              }
              match key.code {
                KeyCode::Esc => {
                  preset_save_active = false;
                  preset_save_input.clear();
                  status_tab = BrowseTab::Review;
                  status_at = Instant::now();
                  status_message = "Preset save canceled".to_string();
                }
                KeyCode::Enter => {
                  let name = preset_save_input.trim();
                  status_tab = BrowseTab::Review;
                  status_at = Instant::now();
                  if name.is_empty() {
                    status_message = "Preset name required".to_string();
                  } else {
                    let entry = build_preset_entry_from_selection(
                      config,
                      &selected_theme,
                      current_waybar_selection(&waybar_items, &waybar_state),
                      current_starship_selection(
                        &starship_items,
                        &starship_state,
                        &theme_path,
                      ),
                    );
                    match presets::save_preset(name, entry, config) {
                      Ok(()) => {
                        status_message = "Preset saved".to_string();
                        preset_file = presets::load_presets()?;
                        preset_items = build_preset_items(&preset_file);
                        reset_picker_cache(&mut preset_state);
                        rebuild_filtered(&mut preset_state, &preset_items);
                        select_preset_by_name(&mut preset_state, &preset_items, name);
                      }
                      Err(err) => {
                        status_message = err.to_string();
                      }
                    }
                  }
                  preset_save_active = false;
                  preset_save_input.clear();
                }
                KeyCode::Backspace => {
                  preset_save_input.pop();
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                  preset_save_input.clear();
                }
                KeyCode::Char(ch) => {
                  if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
                  {
                    preset_save_input.push(ch);
                  }
                }
                _ => {}
              }
              if !event::poll(Duration::from_millis(0))? {
                break 'event_loop;
              }
              continue 'event_loop;
            }
            let is_repeat = key.kind == event::KeyEventKind::Repeat;
            if is_repeat {
              if let Some((last_code, last_mod, last_at)) = last_press_key {
                if last_code == key.code && last_mod == key.modifiers {
                  if now.duration_since(last_at) < Duration::from_millis(150) {
                    if !event::poll(Duration::from_millis(0))? {
                      break 'event_loop;
                    }
                    continue 'event_loop;
                  }
                }
              }
              if let Some((last_code, last_mod)) = last_repeat_key {
                if last_code == key.code && last_mod == key.modifiers {
                  if now.duration_since(last_repeat_at) < Duration::from_millis(35) {
                    if !event::poll(Duration::from_millis(0))? {
                      break 'event_loop;
                    }
                    continue 'event_loop;
                  }
                }
              }
              last_repeat_key = Some((key.code, key.modifiers));
              last_repeat_at = now;
            } else {
              last_press_key = Some((key.code, key.modifiers, now));
            }
            if let Some(state) = active_picker_mut(
              tab,
              &mut theme_state,
              &mut waybar_state,
              &mut starship_state,
              &mut preset_state,
            ) {
              if tab != BrowseTab::Review && state.focus == FocusArea::List {
                let mut handled = false;
                match key.code {
                  KeyCode::Backspace => {
                    state.search_query.pop();
                    handled = true;
                  }
                  KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.search_query.clear();
                    handled = true;
                  }
                  KeyCode::Char(ch) => {
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                      && !key.modifiers.contains(KeyModifiers::ALT)
                    {
                      state.search_query.push(ch);
                      handled = true;
                    }
                  }
                  _ => {}
                }
                if handled {
                  rebuild_active_filtered(
                    tab,
                    &mut theme_state,
                    &mut waybar_state,
                    &mut starship_state,
                    &mut preset_state,
                    &theme_items,
                    &waybar_items,
                    &starship_items,
                    &preset_items,
                  );
                  if !event::poll(Duration::from_millis(0))? {
                    break 'event_loop;
                  }
                  continue 'event_loop;
                }
              }
            }
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
              cleanup_terminal(&mut terminal)?;
              return Ok(None);
            }
            if key.code == KeyCode::Tab {
              tab = next_tab(tab);
              clear_kitty_preview(&backend);
              mark_force_clear(
                &mut theme_state,
                &mut waybar_state,
                &mut starship_state,
                &mut preset_state,
              );
              if !event::poll(Duration::from_millis(0))? {
                break 'event_loop;
              }
              continue 'event_loop;
            }
            if key.code == KeyCode::BackTab {
              tab = previous_tab(tab);
              clear_kitty_preview(&backend);
              mark_force_clear(
                &mut theme_state,
                &mut waybar_state,
                &mut starship_state,
                &mut preset_state,
              );
              if !event::poll(Duration::from_millis(0))? {
                break 'event_loop;
              }
              continue 'event_loop;
            }
            if tab == BrowseTab::Review
              && key.modifiers.contains(KeyModifiers::CONTROL)
              && key.code == KeyCode::Char('s')
            {
              preset_save_active = true;
              preset_save_input.clear();
              if !event::poll(Duration::from_millis(0))? {
                break 'event_loop;
              }
              continue 'event_loop;
            }
            if tab == BrowseTab::Review
              && key.modifiers.contains(KeyModifiers::CONTROL)
              && matches!(key.code, KeyCode::Enter | KeyCode::Char('m') | KeyCode::Char('j'))
            {
              let selection = BrowseSelection {
                theme: selected_theme.clone(),
                waybar: current_waybar_selection(&waybar_items, &waybar_state),
                starship: current_starship_selection(
                  &starship_items,
                  &starship_state,
                  &theme_path,
                ),
              };
              cleanup_terminal(&mut terminal)?;
              return Ok(Some(selection));
            }
            if key.code == KeyCode::Enter && tab == BrowseTab::Presets {
              status_tab = tab;
              status_at = Instant::now();
              match apply_preset_to_states(
                config,
                &preset_items,
                &mut preset_state,
                &theme_items,
                &mut theme_state,
                &mut selected_theme,
                &mut theme_path,
                &mut waybar_items,
                &mut waybar_state,
                &mut starship_items,
                &mut starship_state,
              ) {
                Ok(()) => {
                  status_message = "Preset loaded".to_string();
                  tab = BrowseTab::Review;
                }
                Err(err) => {
                  status_message = err.to_string();
                }
              }
              if !event::poll(Duration::from_millis(0))? {
                break 'event_loop;
              }
              continue 'event_loop;
            }
          if key.code == KeyCode::Enter && tab != BrowseTab::Review {
            status_tab = tab;
            status_at = Instant::now();
            status_message = match tab {
              BrowseTab::Theme => "Theme selected".to_string(),
              BrowseTab::Waybar => "Waybar selected".to_string(),
              BrowseTab::Starship => "Starship selected".to_string(),
              BrowseTab::Presets => "Preset selected".to_string(),
              BrowseTab::Review => String::new(),
            };
            tab = next_tab(tab);
            let items_len = match tab {
              BrowseTab::Theme => theme_state.filtered_indices.len(),
              BrowseTab::Waybar => waybar_state.filtered_indices.len(),
              BrowseTab::Starship => starship_state.filtered_indices.len(),
              BrowseTab::Presets => preset_state.filtered_indices.len(),
              BrowseTab::Review => 0,
            };
            if let Some(state) = active_picker_mut(
              tab,
              &mut theme_state,
              &mut waybar_state,
              &mut starship_state,
              &mut preset_state,
            ) {
              if items_len > 0 {
                state.list_state.select(Some(0));
              } else {
                state.list_state.select(None);
              }
              state.focus = FocusArea::List;
            }
            if !event::poll(Duration::from_millis(0))? {
              break 'event_loop;
            }
            continue 'event_loop;
          }

            let items_len = match tab {
              BrowseTab::Theme => theme_state.filtered_indices.len(),
              BrowseTab::Waybar => waybar_state.filtered_indices.len(),
              BrowseTab::Starship => starship_state.filtered_indices.len(),
              BrowseTab::Presets => preset_state.filtered_indices.len(),
              BrowseTab::Review => 0,
            };
            if let Some(state) = active_picker_mut(
              tab,
              &mut theme_state,
              &mut waybar_state,
              &mut starship_state,
              &mut preset_state,
            ) {
              match key.code {
                KeyCode::Up => match state.focus {
                  FocusArea::List => {
                    let new_index = previous_index(state.list_state.selected(), items_len);
                    state.list_state.select(Some(new_index));
                  }
                  FocusArea::Code => {
                    state.code_scroll = state.code_scroll.saturating_sub(1);
                  }
                },
                KeyCode::Down => match state.focus {
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
                  mark_force_clear(
                    &mut theme_state,
                    &mut waybar_state,
                    &mut starship_state,
                    &mut preset_state,
                  );
                  if !event::poll(Duration::from_millis(0))? {
                    break 'event_loop;
                  }
                  continue 'event_loop;
                }
              }

              let items_len = match tab {
                BrowseTab::Theme => theme_state.filtered_indices.len(),
                BrowseTab::Waybar => waybar_state.filtered_indices.len(),
                BrowseTab::Starship => starship_state.filtered_indices.len(),
                BrowseTab::Presets => preset_state.filtered_indices.len(),
                BrowseTab::Review => 0,
              };
              if let Some(state) = active_picker_mut(
                tab,
                &mut theme_state,
                &mut waybar_state,
                &mut starship_state,
                &mut preset_state,
              ) {
                let position = Position {
                  x: mouse.column,
                  y: mouse.row,
                };
                if active_search_area.contains(position) {
                  state.focus = FocusArea::List;
                } else if active_list_inner.contains(position) {
                  state.focus = FocusArea::List;
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
              let items_len = match tab {
                BrowseTab::Theme => theme_state.filtered_indices.len(),
                BrowseTab::Waybar => waybar_state.filtered_indices.len(),
                BrowseTab::Starship => starship_state.filtered_indices.len(),
                BrowseTab::Presets => preset_state.filtered_indices.len(),
                BrowseTab::Review => 0,
              };
              if let Some(state) = active_picker_mut(
                tab,
                &mut theme_state,
                &mut waybar_state,
                &mut starship_state,
                &mut preset_state,
              ) {
                let position = Position {
                  x: mouse.column,
                  y: mouse.row,
                };
                if active_list_inner.contains(position) {
                  state.focus = FocusArea::List;
                  let new_index = previous_index(state.list_state.selected(), items_len);
                  state.list_state.select(Some(new_index));
                } else if active_code_inner.contains(position) {
                  state.focus = FocusArea::Code;
                  state.code_scroll = state.code_scroll.saturating_sub(1);
                }
              }
            }
            MouseEventKind::ScrollDown => {
              let items_len = match tab {
                BrowseTab::Theme => theme_state.filtered_indices.len(),
                BrowseTab::Waybar => waybar_state.filtered_indices.len(),
                BrowseTab::Starship => starship_state.filtered_indices.len(),
                BrowseTab::Presets => preset_state.filtered_indices.len(),
                BrowseTab::Review => 0,
              };
              if let Some(state) = active_picker_mut(
                tab,
                &mut theme_state,
                &mut waybar_state,
                &mut starship_state,
                &mut preset_state,
              ) {
                let position = Position {
                  x: mouse.column,
                  y: mouse.row,
                };
                if active_list_inner.contains(position) {
                  state.focus = FocusArea::List;
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
        if !event::poll(Duration::from_millis(0))? {
          break;
        }
      }
    }

    if let Some(new_theme) = current_theme_value(&theme_items, &theme_state) {
      if new_theme != selected_theme {
        selected_theme = new_theme;
        theme_path = theme_ops::resolve_theme_path(config, &selected_theme)?;
        let waybar_key = selected_item_key(&waybar_items, &waybar_state);
        let starship_key = selected_item_key(&starship_items, &starship_state);

        waybar_items = build_waybar_items(config, &theme_path)?;
        starship_items = build_starship_items(config, &theme_path)?;

        reset_picker_cache(&mut waybar_state);
        reset_picker_cache(&mut starship_state);

        rebuild_filtered(&mut waybar_state, &waybar_items);
        rebuild_filtered(&mut starship_state, &starship_items);
        select_item_by_key(&mut waybar_state, &waybar_items, waybar_key);
        select_item_by_key(&mut starship_state, &starship_items, starship_key);
        ensure_selected(&mut waybar_state.list_state, waybar_state.filtered_indices.len());
        ensure_selected(
          &mut starship_state.list_state,
          starship_state.filtered_indices.len(),
        );
      }
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

struct PresetItem {
  label: String,
  name: String,
}

fn build_waybar_items(config: &ResolvedConfig, theme_path: &Path) -> Result<Vec<LabeledItem>> {
  let mut items = Vec::new();
  items.push(OptionItem::with_kind(
    "Omarchy default".to_string(),
    "default".to_string(),
    "default",
    None,
  ));
  items.push(OptionItem::with_kind(
    "No waybar changes".to_string(),
    "none".to_string(),
    "none",
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
  items.push(OptionItem::with_kind(
    "No Starship changes".to_string(),
    "none".to_string(),
    "none",
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

fn build_preset_items(file: &presets::PresetFile) -> Vec<PresetItem> {
  let mut names: Vec<String> = file.preset.keys().cloned().collect();
  names.sort();
  names
    .into_iter()
    .map(|name| PresetItem {
      label: name.clone(),
      name,
    })
    .collect()
}

fn build_waybar_code_preview(
  config: &ResolvedConfig,
  theme_path: &Path,
  item: &LabeledItem,
) -> Text<'static> {
  match item.kind.as_str() {
    "default" => Text::from("Using Omarchy default Waybar config."),
    "none" => Text::from("No Waybar changes."),
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
    "none" => Text::from("No Starship config change."),
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
  if item.kind.as_str() == "default" || item.kind.as_str() == "none" {
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
  search_area: Rect,
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
    .constraints([Constraint::Percentage(25), Constraint::Length(1), Constraint::Min(0)].as_ref())
    .split(chunks[0]);
  let image_area = inner_rect(chunks[1]);
  let list_column = top_chunks[0];
  let list_chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
    .split(list_column);
  let search_area = list_chunks[0];
  let list_area = list_chunks[1];
  let list_inner = list_inner_rect(list_area);
  let code_area = top_chunks[2];
  let code_inner = inner_rect(code_area);

  render_search_input(
    frame,
    search_area,
    &state.search_query,
    state.focus == FocusArea::List,
  );

  let list_items: Vec<ListItem> = state
    .filtered_indices
    .iter()
    .map(|&idx| ListItem::new(Line::from(items[idx].label())))
    .collect();
  let list_title = build_list_title(title, status);
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

  let selected = selected_index(&state.list_state, state.filtered_indices.len());
  let selected_item = state.filtered_indices.get(selected).copied();
  let preview_path = selected_item.and_then(|idx| image_preview(idx));

  if let Some(item_index) = selected_item {
    state.last_selected = Some(item_index);
    if Some(item_index) != state.last_code_index {
      state.last_code = code_preview(item_index);
      state.code_scroll = 0;
      state.last_code_index = Some(item_index);
    }
  } else {
    state.last_code = Text::from("No matches.");
    state.last_preview_text = Text::from("No matches.");
    state.last_preview = None;
    state.preview_dirty = true;
    state.last_code_index = None;
    state.last_preview_index = None;
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

  if let Some(item_index) = selected_item {
    if Some(item_index) != state.last_preview_index {
      if let Some(text) = preview_text(item_index) {
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
      state.last_preview_index = Some(item_index);
    }
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
    search_area,
    list_inner,
    code_inner,
    code_area,
  }
}

fn render_preset_picker(
  frame: &mut Frame,
  area: Rect,
  items: &[PresetItem],
  state: &mut PickerState,
  summary: impl Fn(usize) -> Text<'static>,
  status: Option<&str>,
) -> PickerAreas {
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(30), Constraint::Length(1), Constraint::Min(0)].as_ref())
    .split(area);
  let list_column = chunks[0];
  let list_chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
    .split(list_column);
  let search_area = list_chunks[0];
  let list_area = list_chunks[1];
  let list_inner = list_inner_rect(list_area);
  let summary_area = chunks[2];
  let summary_inner = inner_rect(summary_area);

  render_search_input(
    frame,
    search_area,
    &state.search_query,
    state.focus == FocusArea::List,
  );

  let list_items: Vec<ListItem> = state
    .filtered_indices
    .iter()
    .map(|&idx| ListItem::new(Line::from(items[idx].label())))
    .collect();
  let list_title = build_list_title("Select preset", status);
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

  let selected = selected_index(&state.list_state, state.filtered_indices.len());
  let selected_item = state.filtered_indices.get(selected).copied();
  if let Some(item_index) = selected_item {
    state.last_selected = Some(item_index);
    if Some(item_index) != state.last_code_index {
      state.last_code = summary(item_index);
      state.code_scroll = 0;
      state.last_code_index = Some(item_index);
    }
  } else {
    state.last_code = Text::from("No presets found.");
    state.last_code_index = None;
  }

  let max_scroll = state
    .last_code
    .lines
    .len()
    .saturating_sub(summary_inner.height.max(1) as usize);
  if state.code_scroll as usize > max_scroll {
    state.code_scroll = max_scroll as u16;
  }
  let summary_block = Block::default()
    .title("Preset Summary")
    .borders(Borders::ALL)
    .border_style(if state.focus == FocusArea::Code {
      Style::default().fg(Color::Yellow)
    } else {
      Style::default()
    });
  let summary_panel = Paragraph::new(state.last_code.clone())
    .block(summary_block)
    .scroll((state.code_scroll, 0))
    .wrap(Wrap { trim: false });
  frame.render_widget(summary_panel, summary_area);

  PickerAreas {
    search_area,
    list_inner,
    code_inner: summary_inner,
    code_area: summary_area,
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

fn render_status_bar(
  frame: &mut Frame,
  area: Rect,
  tab: BrowseTab,
  theme: &str,
  waybar: String,
  starship: String,
  status: Option<&str>,
  save_active: bool,
  save_input: &str,
) {
  let mut spans = Vec::new();
  let tab_label = match tab {
    BrowseTab::Theme => "Theme",
    BrowseTab::Waybar => "Waybar",
    BrowseTab::Starship => "Starship",
    BrowseTab::Presets => "Presets",
    BrowseTab::Review => "Review",
  };

  push_status_segment(&mut spans, tab_label, Color::Black, Color::Yellow);
  push_status_sep(&mut spans);
  push_status_segment(
    &mut spans,
    &format!("Theme: {}", title_case_theme(theme)),
    Color::Black,
    Color::Cyan,
  );
  push_status_sep(&mut spans);
  push_status_segment(&mut spans, &format!("Waybar: {waybar}"), Color::Black, Color::Green);
  push_status_sep(&mut spans);
  push_status_segment(
    &mut spans,
    &format!("Starship: {starship}"),
    Color::Black,
    Color::Magenta,
  );

  if tab == BrowseTab::Review && !save_active {
    push_status_sep(&mut spans);
    push_status_segment(
      &mut spans,
      "Ctrl+Enter Apply",
      Color::Black,
      Color::LightYellow,
    );
    push_status_sep(&mut spans);
    push_status_segment(
      &mut spans,
      "Ctrl+S Save Preset",
      Color::Black,
      Color::LightYellow,
    );
  }

  if save_active {
    let cursor = "_";
    push_status_sep(&mut spans);
    push_status_segment(
      &mut spans,
      &format!("Save preset: {save_input}{cursor}"),
      Color::Black,
      Color::Blue,
    );
  }

  if let Some(message) = status {
    push_status_sep(&mut spans);
    push_status_segment(&mut spans, message, Color::Black, Color::LightBlue);
  }

  let line = Line::from(spans);
  let bar = Paragraph::new(line);
  frame.render_widget(bar, area);
}

fn push_status_segment(spans: &mut Vec<Span<'static>>, label: &str, fg: Color, bg: Color) {
  spans.push(Span::styled(
    format!(" {} ", label),
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
  ));
}

fn push_status_sep(spans: &mut Vec<Span<'static>>) {
  spans.push(Span::styled(">", Style::default().fg(Color::DarkGray)));
}

fn preset_summary_text(
  config: &ResolvedConfig,
  file: &presets::PresetFile,
  item: &PresetItem,
) -> Text<'static> {
  let entry = match file.preset.get(&item.name) {
    Some(entry) => entry,
    None => return Text::from("Preset not found."),
  };
  let summary = presets::summarize_preset(config, &item.name, entry);
  let mut lines = vec![
    Line::from(format!("Preset: {}", item.name)),
    Line::from(""),
    Line::from(format!("Theme: {}", summary.theme)),
    Line::from(format!("Waybar: {}", summary.waybar)),
    Line::from(format!("Starship: {}", summary.starship)),
  ];
  if !summary.errors.is_empty() {
    lines.push(Line::from(""));
    lines.push(Line::from("Issues:"));
    for err in summary.errors {
      lines.push(Line::from(format!("- {}", err)));
    }
  }
  Text::from(lines)
}

fn render_tab_bar(
  frame: &mut Frame,
  area: Rect,
  titles: &[&str],
  active: BrowseTab,
  ranges: &mut Vec<(u16, u16, usize)>,
) {
  ranges.clear();
  let block = Block::default().borders(Borders::BOTTOM);
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

  let title_label = format!(" {} ", APP_TITLE);
  let title_width = title_label.len() as u16;
  let sep_width = 1u16;
  let used_width = cursor.saturating_sub(inner.x);
  let total_needed = used_width
    .saturating_add(sep_width)
    .saturating_add(title_width);
  let spacer_len = if inner.width > total_needed {
    (inner.width - total_needed) as usize
  } else {
    1
  };
  spans.push(Span::raw(" ".repeat(spacer_len)));
  spans.push(Span::raw("│"));
  spans.push(Span::styled(
    title_label,
    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
  ));

  let line = Line::from(spans);
  let tabs = Paragraph::new(line).alignment(ratatui::layout::Alignment::Left);
  frame.render_widget(tabs, inner);
}

fn build_list_title(title: &str, status: Option<&str>) -> String {
  let out = match status {
    Some(status) => format!("{title}  [{status}]"),
    None => title.to_string(),
  };
  out
}

fn render_search_input(frame: &mut Frame, area: Rect, query: &str, focused: bool) {
  let (content, style) = if query.is_empty() {
    ("󰍉 Search...".to_string(), Style::default().fg(Color::DarkGray))
  } else {
    (format!("󰍉 {}", query), Style::default())
  };
  let block = Block::default()
    .title("Search")
    .borders(Borders::ALL)
    .border_style(if focused {
      Style::default().fg(Color::Yellow)
    } else {
      Style::default()
    });
  let input = Paragraph::new(Line::from(Span::styled(content, style))).block(block);
  frame.render_widget(input, area);
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
  presets: &mut PickerState,
) {
  theme.force_clear = true;
  waybar.force_clear = true;
  starship.force_clear = true;
  presets.force_clear = true;
}

fn active_picker_mut<'a>(
  tab: BrowseTab,
  theme: &'a mut PickerState,
  waybar: &'a mut PickerState,
  starship: &'a mut PickerState,
  presets: &'a mut PickerState,
) -> Option<&'a mut PickerState> {
  match tab {
    BrowseTab::Theme => Some(theme),
    BrowseTab::Waybar => Some(waybar),
    BrowseTab::Starship => Some(starship),
    BrowseTab::Presets => Some(presets),
    BrowseTab::Review => None,
  }
}

fn rebuild_active_filtered(
  tab: BrowseTab,
  theme: &mut PickerState,
  waybar: &mut PickerState,
  starship: &mut PickerState,
  presets: &mut PickerState,
  theme_items: &[OptionItem],
  waybar_items: &[LabeledItem],
  starship_items: &[LabeledItem],
  preset_items: &[PresetItem],
) {
  match tab {
    BrowseTab::Theme => rebuild_filtered(theme, theme_items),
    BrowseTab::Waybar => rebuild_filtered(waybar, waybar_items),
    BrowseTab::Starship => rebuild_filtered(starship, starship_items),
    BrowseTab::Presets => rebuild_filtered(presets, preset_items),
    BrowseTab::Review => {}
  }
}

fn tab_index(tab: BrowseTab) -> usize {
  match tab {
    BrowseTab::Theme => 0,
    BrowseTab::Waybar => 1,
    BrowseTab::Starship => 2,
    BrowseTab::Review => 3,
    BrowseTab::Presets => 4,
  }
}

fn tab_from_index(index: usize) -> BrowseTab {
  match index {
    0 => BrowseTab::Theme,
    1 => BrowseTab::Waybar,
    2 => BrowseTab::Starship,
    3 => BrowseTab::Review,
    _ => BrowseTab::Presets,
  }
}

fn next_tab(tab: BrowseTab) -> BrowseTab {
  tab_from_index((tab_index(tab) + 1) % 5)
}

fn previous_tab(tab: BrowseTab) -> BrowseTab {
  tab_from_index((tab_index(tab) + 4) % 5)
}

fn tab_index_from_click(ranges: &[(u16, u16, usize)], column: u16) -> Option<usize> {
  ranges
    .iter()
    .find(|(start, end, _)| column >= *start && column <= *end)
    .map(|(_, _, idx)| *idx)
}

fn current_theme_value(items: &[OptionItem], state: &PickerState) -> Option<String> {
  let index = selected_item_index(state, items.len())?;
  Some(items[index].value.clone())
}

fn current_preset_name(items: &[PresetItem], state: &PickerState) -> Option<String> {
  let index = selected_item_index(state, items.len())?;
  Some(items[index].name.clone())
}

fn select_option_by_value(state: &mut PickerState, items: &[OptionItem], value: &str) -> bool {
  if let Some(item_index) = items.iter().position(|item| item.value == value) {
    if let Some(filtered_pos) = state
      .filtered_indices
      .iter()
      .position(|&idx| idx == item_index)
    {
      state.list_state.select(Some(filtered_pos));
      state.last_selected = Some(item_index);
      return true;
    }
  }
  false
}

fn select_preset_by_name(state: &mut PickerState, items: &[PresetItem], name: &str) -> bool {
  if let Some(item_index) = items.iter().position(|item| item.name == name) {
    if let Some(filtered_pos) = state
      .filtered_indices
      .iter()
      .position(|&idx| idx == item_index)
    {
      state.list_state.select(Some(filtered_pos));
      state.last_selected = Some(item_index);
      return true;
    }
  }
  false
}

fn preset_waybar_key(preset: &presets::PresetDefinition) -> Option<(String, String)> {
  match &preset.waybar {
    presets::PresetWaybarValue::None => Some(("none".to_string(), "none".to_string())),
    presets::PresetWaybarValue::Auto => Some(("theme".to_string(), "theme".to_string())),
    presets::PresetWaybarValue::Named(name) => Some(("named".to_string(), name.clone())),
  }
}

fn preset_starship_key(preset: &presets::PresetDefinition) -> Option<(String, String)> {
  match &preset.starship {
    presets::PresetStarshipValue::None => Some(("none".to_string(), "none".to_string())),
    presets::PresetStarshipValue::Theme => Some(("theme".to_string(), "theme".to_string())),
    presets::PresetStarshipValue::Preset(name) => Some(("preset".to_string(), name.clone())),
    presets::PresetStarshipValue::Named(name) => Some(("named".to_string(), name.clone())),
  }
}

fn apply_preset_to_states(
  config: &ResolvedConfig,
  preset_items: &[PresetItem],
  preset_state: &mut PickerState,
  theme_items: &[OptionItem],
  theme_state: &mut PickerState,
  selected_theme: &mut String,
  theme_path: &mut PathBuf,
  waybar_items: &mut Vec<LabeledItem>,
  waybar_state: &mut PickerState,
  starship_items: &mut Vec<LabeledItem>,
  starship_state: &mut PickerState,
) -> Result<()> {
  let name = current_preset_name(preset_items, preset_state)
    .ok_or_else(|| anyhow!("no preset selected"))?;

  let preset = presets::load_preset_definition(config, &name)?;
  let normalized = normalize_theme_name(&preset.theme);
  let mut applied_theme = normalized.clone();
  if !select_option_by_value(theme_state, theme_items, &normalized) {
    if select_option_by_value(theme_state, theme_items, &preset.theme) {
      applied_theme = preset.theme.clone();
    } else {
      return Err(anyhow!("preset theme not found in theme list"));
    }
  }

  *selected_theme = applied_theme.clone();
  *theme_path = theme_ops::resolve_theme_path(config, &applied_theme)?;

  *waybar_items = build_waybar_items(config, theme_path)?;
  *starship_items = build_starship_items(config, theme_path)?;
  reset_picker_cache(waybar_state);
  reset_picker_cache(starship_state);
  rebuild_filtered(waybar_state, waybar_items);
  rebuild_filtered(starship_state, starship_items);

  select_item_by_key(waybar_state, waybar_items, preset_waybar_key(&preset));
  select_item_by_key(
    starship_state,
    starship_items,
    preset_starship_key(&preset),
  );
  ensure_selected(&mut waybar_state.list_state, waybar_state.filtered_indices.len());
  ensure_selected(
    &mut starship_state.list_state,
    starship_state.filtered_indices.len(),
  );

  Ok(())
}

fn build_preset_entry_from_selection(
  config: &ResolvedConfig,
  theme: &str,
  waybar_selection: WaybarSelection,
  starship_selection: StarshipSelection,
) -> presets::PresetEntry {
  let waybar_entry = match waybar_selection {
    WaybarSelection::UseDefaults => waybar_entry_from_defaults(config),
    WaybarSelection::None => presets::PresetWaybarEntry {
      mode: Some("none".to_string()),
      name: None,
    },
    WaybarSelection::Auto => presets::PresetWaybarEntry {
      mode: Some("auto".to_string()),
      name: None,
    },
    WaybarSelection::Named(name) => presets::PresetWaybarEntry {
      mode: Some("named".to_string()),
      name: Some(name),
    },
  };

  let starship_entry = match starship_selection {
    StarshipSelection::UseDefaults => starship_entry_from_defaults(config),
    StarshipSelection::None => presets::PresetStarshipEntry {
      mode: Some("none".to_string()),
      preset: None,
      name: None,
    },
    StarshipSelection::Preset(preset) => presets::PresetStarshipEntry {
      mode: Some("preset".to_string()),
      preset: Some(preset),
      name: None,
    },
    StarshipSelection::Named(name) => presets::PresetStarshipEntry {
      mode: Some("named".to_string()),
      preset: None,
      name: Some(name),
    },
    StarshipSelection::Theme(_) => presets::PresetStarshipEntry {
      mode: Some("theme".to_string()),
      preset: None,
      name: None,
    },
  };

  presets::PresetEntry {
    theme: Some(theme.to_string()),
    waybar: Some(waybar_entry),
    starship: Some(starship_entry),
  }
}

fn waybar_entry_from_defaults(config: &ResolvedConfig) -> presets::PresetWaybarEntry {
  match waybar_from_defaults(config) {
    (WaybarMode::Auto, _) => presets::PresetWaybarEntry {
      mode: Some("auto".to_string()),
      name: None,
    },
    (WaybarMode::Named, Some(name)) => presets::PresetWaybarEntry {
      mode: Some("named".to_string()),
      name: Some(name),
    },
    _ => presets::PresetWaybarEntry {
      mode: Some("none".to_string()),
      name: None,
    },
  }
}

fn starship_entry_from_defaults(config: &ResolvedConfig) -> presets::PresetStarshipEntry {
  match starship_from_defaults(config) {
    StarshipMode::Preset { preset } => presets::PresetStarshipEntry {
      mode: Some("preset".to_string()),
      preset: Some(preset),
      name: None,
    },
    StarshipMode::Named { name } => presets::PresetStarshipEntry {
      mode: Some("named".to_string()),
      preset: None,
      name: Some(name),
    },
    _ => presets::PresetStarshipEntry {
      mode: Some("none".to_string()),
      preset: None,
      name: None,
    },
  }
}

fn current_waybar_label(items: &[LabeledItem], state: &PickerState) -> String {
  let index = match selected_item_index(state, items.len()) {
    Some(index) => index,
    None => return "No options".to_string(),
  };
  if items.len() == 1 && items[0].kind == "default" {
    return "Use defaults".to_string();
  }
  let item = &items[index];
  match item.kind.as_str() {
    "default" => "Omarchy default".to_string(),
    "theme" => "Theme waybar".to_string(),
    "none" => "No waybar changes".to_string(),
    _ => item.label.clone(),
  }
}

fn current_starship_label(items: &[LabeledItem], state: &PickerState) -> String {
  let index = match selected_item_index(state, items.len()) {
    Some(index) => index,
    None => return "No options".to_string(),
  };
  if items.len() == 1 && items[0].kind == "default" {
    return "Use defaults".to_string();
  }
  let item = &items[index];
  match item.kind.as_str() {
    "default" => "Omarchy default".to_string(),
    "theme" => "Theme starship".to_string(),
    "none" => "No Starship changes".to_string(),
    _ => item.label.clone(),
  }
}

fn current_waybar_selection(items: &[LabeledItem], state: &PickerState) -> WaybarSelection {
  let index = match selected_item_index(state, items.len()) {
    Some(index) => index,
    None => return WaybarSelection::UseDefaults,
  };
  if items.len() == 1 && items[0].kind == "default" {
    return WaybarSelection::UseDefaults;
  }
  match items[index].kind.as_str() {
    "default" => WaybarSelection::UseDefaults,
    "none" => WaybarSelection::None,
    "theme" => WaybarSelection::Auto,
    _ => WaybarSelection::Named(items[index].value.clone()),
  }
}

fn current_starship_selection(
  items: &[LabeledItem],
  state: &PickerState,
  theme_path: &Path,
) -> StarshipSelection {
  let index = match selected_item_index(state, items.len()) {
    Some(index) => index,
    None => return StarshipSelection::UseDefaults,
  };
  if items.len() == 1 && items[0].kind == "default" {
    return StarshipSelection::UseDefaults;
  }
  match items[index].kind.as_str() {
    "default" => StarshipSelection::UseDefaults,
    "none" => StarshipSelection::None,
    "theme" => StarshipSelection::Theme(theme_path.join("starship.yaml")),
    "preset" => StarshipSelection::Preset(items[index].value.clone()),
    _ => StarshipSelection::Named(items[index].value.clone()),
  }
}

fn selected_item_key(items: &[LabeledItem], state: &PickerState) -> Option<(String, String)> {
  let index = selected_item_index(state, items.len())?;
  Some((items[index].kind.clone(), items[index].value.clone()))
}

fn select_item_by_key(state: &mut PickerState, items: &[LabeledItem], key: Option<(String, String)>) {
  if let Some((kind, value)) = key {
    if let Some(item_index) = items
      .iter()
      .position(|item| item.kind == kind && item.value == value)
    {
      if let Some(filtered_pos) = state
        .filtered_indices
        .iter()
        .position(|&idx| idx == item_index)
      {
        state.list_state.select(Some(filtered_pos));
        state.last_selected = Some(item_index);
      }
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

fn filter_item_indices<T: ItemView>(items: &[T], query: &str) -> Vec<usize> {
  if query.trim().is_empty() {
    return (0..items.len()).collect();
  }
  let mut scored: Vec<(i64, usize, String)> = Vec::new();
  for (idx, item) in items.iter().enumerate() {
    let label = item.label();
    if let Some(score) = fuzzy_score(&label, query) {
      scored.push((score, idx, label));
    }
  }
  scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.2.cmp(&b.2)));
  scored.into_iter().map(|(_, idx, _)| idx).collect()
}

fn fuzzy_score(label: &str, query: &str) -> Option<i64> {
  let query = query.trim();
  if query.is_empty() {
    return None;
  }
  let label_lower = label.to_lowercase();
  let query_lower = query.to_lowercase();
  let label_chars: Vec<char> = label_lower.chars().collect();
  let query_chars: Vec<char> = query_lower.chars().collect();
  let qlen = query_chars.len();

  let mut score = 0i64;
  let contains_pos = label_lower.find(&query_lower);
  if let Some(pos) = contains_pos {
    score += 20_000;
    score += (5000 - pos as i64).max(0);
    if pos == 0 {
      score += 8000;
    } else if is_word_boundary(&label_chars, pos) {
      score += 2000;
    }
  }

  let mut positions: Vec<usize> = Vec::with_capacity(query_chars.len());
  let mut q = 0;
  for (i, ch) in label_chars.iter().enumerate() {
    if *ch == query_chars[q] {
      positions.push(i);
      q += 1;
      if q == query_chars.len() {
        break;
      }
    }
  }
  if q != query_chars.len() {
    return if score > 0 { Some(score) } else { None };
  }

  score += 2000;
  if positions.first() == Some(&0) {
    score += 1500;
  } else if let Some(first) = positions.first().copied() {
    if is_word_boundary(&label_chars, first) {
      score += 500;
    }
  }
  for window in positions.windows(2) {
    let prev = window[0];
    let next = window[1];
    if next == prev + 1 {
      score += 400;
    } else {
      score -= (next - prev) as i64 * 2;
    }
  }
  if qlen <= 2 && contains_pos.is_none() {
    score -= 5000;
  }
  score += 500 - label_chars.len() as i64;
  Some(score)
}

fn is_word_boundary(chars: &[char], idx: usize) -> bool {
  if idx == 0 {
    return true;
  }
  !chars[idx.saturating_sub(1)].is_alphanumeric()
}

fn selected_item_index(state: &PickerState, len: usize) -> Option<usize> {
  let idx = if !state.filtered_indices.is_empty() {
    let selected = selected_index(&state.list_state, state.filtered_indices.len());
    state.filtered_indices.get(selected).copied()
  } else {
    state.last_selected
  };
  match idx {
    Some(idx) if idx < len => Some(idx),
    _ => None,
  }
}

fn rebuild_filtered<T: ItemView>(state: &mut PickerState, items: &[T]) {
  let previous = selected_item_index(state, items.len());
  state.filtered_indices = filter_item_indices(items, &state.search_query);
  let query_changed = state.search_query != state.last_query;
  state.last_query = state.search_query.clone();
  if query_changed && !state.search_query.trim().is_empty() {
    ensure_selected(&mut state.list_state, state.filtered_indices.len());
    if let Some(selected) = state.filtered_indices.first().copied() {
      state.list_state.select(Some(0));
      state.last_selected = Some(selected);
    }
    return;
  }
  if let Some(item_index) = previous {
    if let Some(pos) = state
      .filtered_indices
      .iter()
      .position(|&idx| idx == item_index)
    {
      state.list_state.select(Some(pos));
      state.last_selected = Some(item_index);
      return;
    }
  }
  ensure_selected(&mut state.list_state, state.filtered_indices.len());
  if let Some(selected) = state
    .filtered_indices
    .get(selected_index(&state.list_state, state.filtered_indices.len()))
    .copied()
  {
    state.last_selected = Some(selected);
  }
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

impl ItemView for PresetItem {
  fn label(&self) -> String {
    self.label.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct DummyItem {
    label: String,
  }

  impl ItemView for DummyItem {
    fn label(&self) -> String {
      self.label.clone()
    }
  }

  #[test]
  fn filter_items_empty_query_returns_all() {
    let items = vec![
      DummyItem {
        label: "alpha".to_string(),
      },
      DummyItem {
        label: "bravo".to_string(),
      },
      DummyItem {
        label: "charlie".to_string(),
      },
    ];
    let filtered = filter_item_indices(&items, "");
    assert_eq!(filtered, vec![0, 1, 2]);
  }

  #[test]
  fn filter_items_with_query_returns_matches() {
    let items = vec![
      DummyItem {
        label: "alpha".to_string(),
      },
      DummyItem {
        label: "bravo".to_string(),
      },
      DummyItem {
        label: "charlie".to_string(),
      },
    ];
    let filtered = filter_item_indices(&items, "br");
    assert_eq!(filtered, vec![1]);
  }

  #[test]
  fn rebuild_filtered_preserves_last_selected() {
    let items = vec![
      DummyItem {
        label: "alpha".to_string(),
      },
      DummyItem {
        label: "bravo".to_string(),
      },
    ];
    let mut state = PickerState::new();
    rebuild_filtered(&mut state, &items);
    state.list_state.select(Some(1));
    rebuild_filtered(&mut state, &items);
    assert_eq!(state.last_selected, Some(1));

    state.search_query = "zzz".to_string();
    rebuild_filtered(&mut state, &items);
    assert!(state.filtered_indices.is_empty());
    assert_eq!(state.last_selected, Some(1));
  }

  #[test]
  fn filter_items_falls_back_to_substring_match() {
    let items = vec![
      DummyItem {
        label: "dracula".to_string(),
      },
      DummyItem {
        label: "nord".to_string(),
      },
    ];
    let filtered = filter_item_indices(&items, "dra");
    assert_eq!(filtered, vec![0]);
  }

  #[test]
  fn filter_items_supports_subsequence_match() {
    let items = vec![
      DummyItem {
        label: "dracula".to_string(),
      },
      DummyItem {
        label: "nord".to_string(),
      },
    ];
    let filtered = filter_item_indices(&items, "drc");
    assert_eq!(filtered, vec![0]);
  }

  #[test]
  fn preset_keys_map_to_items() {
    let preset = presets::PresetDefinition {
      name: "Test".to_string(),
      theme: "noir".to_string(),
      waybar: presets::PresetWaybarValue::None,
      starship: presets::PresetStarshipValue::Theme,
    };
    assert_eq!(
      preset_waybar_key(&preset),
      Some(("none".to_string(), "none".to_string()))
    );
    assert_eq!(
      preset_starship_key(&preset),
      Some(("theme".to_string(), "theme".to_string()))
    );
  }
}
