use crate::config::load_config;
use crate::model::{Alias, Platform, Profile};
use crate::paths::AppPaths;
use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use std::io;
use std::time::Duration;

pub fn run(paths: &AppPaths, profile: Option<Profile>) -> Result<()> {
    let result = (|| -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut app = App::load(paths, profile)?;

        event_loop(&mut terminal, &mut app, paths)
    })();

    cleanup_terminal();
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    paths: &AppPaths,
) -> Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Home | KeyCode::Char('g') => app.first(),
                    KeyCode::End | KeyCode::Char('G') => app.last(),
                    KeyCode::Char('r') => {
                        if let Err(error) = app.reload(paths) {
                            app.status = format!("reload failed: {error}");
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut ratatui::Frame<'_>, app: &App) {
    let root = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(root);

    draw_header(frame, layout[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(layout[1]);

    draw_list(frame, body[0], app);
    draw_details(frame, body[1], app);
    draw_footer(frame, layout[2], app);
}

fn draw_header(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let profile = app
        .profile
        .map(|profile| format_profile(profile))
        .unwrap_or_else(|| "all profiles".to_string());
    let text = Paragraph::new(format!(
        "HunMing TUI  |  {} aliases  |  {}  |  {}",
        app.entries.len(),
        profile,
        app.paths.config_file.display()
    ))
    .block(Block::default().borders(Borders::ALL).title("Status"));

    frame.render_widget(text, area);
}

fn draw_list(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let items: Vec<ListItem> = if app.entries.is_empty() {
        vec![ListItem::new("No aliases configured.")]
    } else {
        app.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let selected = app.selected == Some(index);
                let marker = if selected { ">" } else { " " };
                let status_style = if entry.is_active {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let name_style = if selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    status_style
                };

                let tags = if entry.alias.tags.is_empty() {
                    "-".to_string()
                } else {
                    entry.alias.tags.join(", ")
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{marker} "), status_style),
                    Span::styled(entry.name.clone(), name_style),
                    Span::raw(format!("  [{tags}]")),
                ]))
            })
            .collect()
    };

    let mut state = ListState::default();
    state.select(app.selected);

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Aliases"))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_details(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let paragraph = match app.selected_entry() {
        Some(entry) => Paragraph::new(alias_details(entry, app.profile))
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: true }),
        None => Paragraph::new("Select an alias to inspect its details.")
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: true }),
    };

    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, app: &App) {
    let text = Paragraph::new(app.status.clone())
        .block(Block::default().borders(Borders::ALL).title("Help"));

    frame.render_widget(text, area);
}

fn alias_details(entry: &AliasEntry, profile: Option<Profile>) -> Text<'static> {
    let mut lines = vec![
        field_line("Name", &entry.name),
        field_line(
            "State",
            if entry.is_active {
                "active"
            } else {
                "inactive"
            },
        ),
        field_line(
            "Current profile",
            profile
                .map(format_profile)
                .unwrap_or_else(|| "none".to_string()),
        ),
        field_line(
            "Profile rule",
            entry
                .alias
                .profile
                .map(format_profile)
                .unwrap_or_else(|| "all".to_string()),
        ),
        field_line("Tags", format_tags(&entry.alias.tags)),
        field_line("Command", format_command(&entry.alias.command)),
        field_line(
            "Forward args",
            if entry.alias.forward_args {
                "yes"
            } else {
                "no"
            },
        ),
        field_line("Platforms", format_platforms(&entry.alias.platforms)),
        field_line(
            "Platform match",
            if entry.platform_match { "yes" } else { "no" },
        ),
        field_line(
            "Profile match",
            if entry.profile_match { "yes" } else { "no" },
        ),
    ];

    if let Some(description) = entry.alias.description.as_ref() {
        lines.push(field_line("Description", description));
    }

    if let Some(bash) = entry.alias.bash.as_ref() {
        lines.push(field_line("Bash", bash));
    }

    if let Some(powershell) = entry.alias.powershell.as_ref() {
        lines.push(field_line("PowerShell", powershell));
    }

    Text::from(lines)
}

fn field_line(label: &str, value: impl Into<String>) -> Line<'static> {
    let value = value.into();
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value),
    ])
}

fn format_command(command: &[String]) -> String {
    if command.is_empty() {
        "-".to_string()
    } else {
        command.join(" ")
    }
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "-".to_string()
    } else {
        tags.join(", ")
    }
}

fn format_platforms(platforms: &[Platform]) -> String {
    if platforms.is_empty() {
        "all".to_string()
    } else {
        platforms
            .iter()
            .map(|platform| match platform {
                Platform::Windows => "windows",
                Platform::Macos => "macos",
                Platform::Linux => "linux",
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn format_profile(profile: Profile) -> String {
    match profile {
        Profile::Work => "work".to_string(),
        Profile::Personal => "personal".to_string(),
    }
}

fn cleanup_terminal() {
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
}

struct App {
    paths: AppPaths,
    profile: Option<Profile>,
    entries: Vec<AliasEntry>,
    selected: Option<usize>,
    pub status: String,
}

impl App {
    fn load(paths: &AppPaths, profile: Option<Profile>) -> Result<Self> {
        let config = load_config(paths)?;
        let entries = build_entries(&config.aliases, profile);
        let selected = if entries.is_empty() { None } else { Some(0) };

        Ok(Self {
            paths: paths.clone(),
            profile,
            entries,
            selected,
            status: "q/Esc quit  r reload  ↑/↓ move  Home/End jump".to_string(),
        })
    }

    fn reload(&mut self, paths: &AppPaths) -> Result<()> {
        let selected_name = self.selected_entry().map(|entry| entry.name.clone());
        let config = load_config(paths)?;
        self.entries = build_entries(&config.aliases, self.profile);
        self.selected = selected_name
            .and_then(|name| self.entries.iter().position(|entry| entry.name == name))
            .or_else(|| (!self.entries.is_empty()).then_some(0));
        self.status = format!(
            "reloaded {} aliases from {}",
            self.entries.len(),
            paths.config_file.display()
        );
        Ok(())
    }

    fn selected_entry(&self) -> Option<&AliasEntry> {
        self.selected.and_then(|index| self.entries.get(index))
    }

    fn next(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        self.selected = Some(match self.selected {
            Some(index) if index + 1 < self.entries.len() => index + 1,
            _ => 0,
        });
    }

    fn previous(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        self.selected = Some(match self.selected {
            Some(0) | None => self.entries.len() - 1,
            Some(index) => index.saturating_sub(1),
        });
    }

    fn first(&mut self) {
        if !self.entries.is_empty() {
            self.selected = Some(0);
        }
    }

    fn last(&mut self) {
        if !self.entries.is_empty() {
            self.selected = Some(self.entries.len() - 1);
        }
    }
}

fn build_entries(
    aliases: &std::collections::BTreeMap<String, Alias>,
    profile: Option<Profile>,
) -> Vec<AliasEntry> {
    aliases
        .iter()
        .map(|(name, alias)| AliasEntry {
            name: name.clone(),
            is_active: alias.is_active_for_current_platform()
                && alias.is_active_for_profile(profile),
            platform_match: alias.is_active_for_current_platform(),
            profile_match: alias.is_active_for_profile(profile),
            alias: alias.clone(),
        })
        .collect()
}

struct AliasEntry {
    name: String,
    alias: Alias,
    is_active: bool,
    platform_match: bool,
    profile_match: bool,
}

#[cfg(test)]
mod tests {
    use super::build_entries;
    use crate::model::{Alias, Config, Profile};
    use std::collections::BTreeMap;

    #[test]
    fn entries_respect_profile_filter() {
        let mut aliases = BTreeMap::new();
        aliases.insert(
            "gs".to_string(),
            Alias {
                description: None,
                command: vec!["git".into(), "status".into()],
                tags: vec![],
                bash: None,
                powershell: None,
                forward_args: true,
                platforms: vec![],
                profile: Some(Profile::Work),
            },
        );

        let config = Config {
            version: 1,
            aliases,
        };

        let active = build_entries(&config.aliases, Some(Profile::Work));
        assert!(active[0].profile_match);
        assert!(active[0].is_active);

        let inactive = build_entries(&config.aliases, Some(Profile::Personal));
        assert!(!inactive[0].profile_match);
        assert!(!inactive[0].is_active);
    }
}
