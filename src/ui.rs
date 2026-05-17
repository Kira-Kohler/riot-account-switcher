use crate::{
    crypto,
    db::{Account, Database},
    riot,
};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

const RED:   Color = Color::Rgb(255, 70, 85);
const DIM:   Color = Color::Rgb(80, 80, 92);
const MID:   Color = Color::Rgb(155, 155, 168);
const CYAN:  Color = Color::Rgb(95, 205, 255);
const GREEN: Color = Color::Rgb(95, 215, 145);
const WHITE: Color = Color::Rgb(235, 235, 245);
const DARK:  Color = Color::Rgb(60, 18, 22);

const BANNER_K1R4: [&str; 6] = [
    "██╗  ██╗ ██╗██████╗ ██╗  ██╗",
    "██╔ ██╔╝███║██╔══██╗██║  ██║",
    "█████╔╝ ╚██║██████╔╝███████║",
    "██╔═██╗  ██║██╔══██╗╚════██║",
    "██║  ██╗ ██║██║  ██║     ██║",
    "╚═╝  ╚═╝ ╚═╝╚═╝  ╚═╝     ╚═╝",
];

const BANNER_LABS: [&str; 6] = [
    "██╗      █████╗ ██████╗ ███████╗",
    "██║     ██╔══██╗██╔══██╗██╔════╝",
    "██║     ███████║██████╔╝███████╗",
    "██║     ██╔══██║██╔══██╗╚════██║",
    "███████╗██║  ██║██████╔╝███████║",
    "╚══════╝╚═╝  ╚═╝╚═════╝ ╚══════╝",
];

enum Mode {
    Normal,
    SaveInput(String),
    RenameInput(String),
    ConfirmDelete,
    ConfirmLogout,
    ExportPass(String),
    ImportPath(String),
    ImportPass { path: String, pass: String },
}

pub struct App {
    db: Database,
    accounts: Vec<Account>,
    list_state: ListState,
    mode: Mode,
    status: String,
    status_err: bool,
}

impl App {
    pub fn new(db: Database) -> Result<Self> {
        let accounts = db.list()?;
        let mut list_state = ListState::default();
        if !accounts.is_empty() {
            list_state.select(Some(0));
        }
        Ok(Self {
            db,
            accounts,
            list_state,
            mode: Mode::Normal,
            status: "Ready — log into Riot Client, then press [S] to save the session.".into(),
            status_err: false,
        })
    }

    fn refresh(&mut self) {
        match self.db.list() {
            Ok(accs) => {
                let prev = self.list_state.selected().unwrap_or(0);
                self.accounts = accs;
                let clamped = prev.min(self.accounts.len().saturating_sub(1));
                self.list_state
                    .select(if self.accounts.is_empty() { None } else { Some(clamped) });
            }
            Err(e) => self.set_err(format!("DB error: {e}")),
        }
    }

    fn selected(&self) -> Option<&Account> {
        self.list_state.selected().and_then(|i| self.accounts.get(i))
    }

    fn set_ok(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
        self.status_err = false;
    }

    fn set_err(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
        self.status_err = true;
    }

    fn save_session(&mut self, name: &str) {
        match riot::read_tokens() {
            Ok(tokens) => {
                let riot_id = riot::fetch_riot_id_live();
                match self.db.upsert(name, &tokens, riot_id.as_deref()) {
                    Ok(()) => {
                        self.refresh();
                        self.set_ok(format!("Saved session as \"{name}\""));
                    }
                    Err(e) => self.set_err(format!("Could not save: {e}")),
                }
            }
            Err(e) => self.set_err(e.to_string()),
        }
    }

    fn switch_to_selected(&mut self) {
        if let Some(acc) = self.selected() {
            let (name, tokens) = (acc.name.clone(), acc.token_data.clone());
            match riot::switch_to(&tokens) {
                Ok(()) => {
                    self.set_ok(format!("Switched to \"{name}\" — Riot Client is launching..."));
                    let db_path = crate::db::db_path();
                    let name_bg = name.clone();
                    let tokens_bg = tokens.clone();
                    std::thread::spawn(move || {
                        for _ in 0..5 {
                            std::thread::sleep(std::time::Duration::from_secs(3));
                            if let Some(live_id) = riot::fetch_riot_id_live() {
                                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                                    let _ = conn.execute(
                                        "UPDATE accounts SET riot_id = ?1 WHERE name = ?2",
                                        rusqlite::params![live_id, name_bg],
                                    );
                                }
                                break;
                            }
                        }
                        if let Some(extracted) = riot::extract_riot_id(&tokens_bg) {
                            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                                let _ = conn.execute(
                                    "UPDATE accounts SET riot_id = COALESCE(riot_id, ?1) WHERE name = ?2",
                                    rusqlite::params![extracted, name_bg],
                                );
                            }
                        }
                    });
                }
                Err(e) => self.set_err(format!("Switch failed: {e}")),
            }
        }
    }

    fn delete_selected(&mut self) {
        if let Some(acc) = self.selected() {
            let (id, name) = (acc.id, acc.name.clone());
            match self.db.delete(id) {
                Ok(()) => {
                    self.refresh();
                    self.set_ok(format!("Deleted \"{name}\""));
                }
                Err(e) => self.set_err(format!("Delete failed: {e}")),
            }
        }
    }

    fn rename_selected(&mut self, new_name: &str) {
        if let Some(acc) = self.selected() {
            let id = acc.id;
            match self.db.rename(id, new_name) {
                Ok(()) => {
                    self.refresh();
                    self.set_ok(format!("Renamed to \"{new_name}\""));
                }
                Err(e) => self.set_err(format!("Rename failed: {e}")),
            }
        }
    }

    fn update_selected(&mut self) {
        if let Some(acc) = self.selected() {
            let (id, name) = (acc.id, acc.name.clone());
            match riot::read_tokens() {
                Ok(tokens) => {
                    let riot_id = riot::fetch_riot_id_live();
                    match self.db.upsert(&name, &tokens, riot_id.as_deref()) {
                        Ok(()) => {
                            self.refresh();
                            if let Some(idx) = self.accounts.iter().position(|a| a.id == id) {
                                self.list_state.select(Some(idx));
                            }
                            self.set_ok(format!("Updated tokens for \"{name}\""));
                        }
                        Err(e) => self.set_err(format!("Update failed: {e}")),
                    }
                }
                Err(e) => self.set_err(e.to_string()),
            }
        }
    }

    fn export_selected(&mut self, password: &str) {
        if let Some(acc) = self.selected() {
            let name = acc.name.clone();
            let tokens = acc.token_data.clone();
            let riot_id = acc.riot_id.clone();
            match crypto::export(&name, &tokens, riot_id.as_deref(), password) {
                Ok(json) => {
                    let path = crypto::export_path(&name);
                    match std::fs::write(&path, &json) {
                        Ok(()) => self.set_ok(format!("Exported to: {}", path.display())),
                        Err(e) => self.set_err(format!("Write failed: {e}")),
                    }
                }
                Err(e) => self.set_err(format!("Export failed: {e}")),
            }
        }
    }

    fn import_account(&mut self, path_str: &str, password: &str) {
        let path = crypto::resolve_import_path(path_str);
        match std::fs::read_to_string(&path) {
            Ok(json) => match crypto::import(&json, password) {
                Ok((name, tokens, payload_riot_id)) => {
                    let riot_id = payload_riot_id.or_else(|| riot::extract_riot_id(&tokens));
                    match self.db.upsert(&name, &tokens, riot_id.as_deref()) {
                        Ok(()) => {
                            self.refresh();
                            self.set_ok(format!("Imported \"{name}\" successfully"));
                        }
                        Err(e) => self.set_err(format!("DB error: {e}")),
                    }
                }
                Err(e) => self.set_err(e.to_string()),
            },
            Err(_) => self.set_err(format!("File not found: {}", path.display())),
        }
    }

    fn scroll_up(&mut self) {
        if self.accounts.is_empty() { return; }
        let n = self.accounts.len();
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(if i == 0 { n - 1 } else { i - 1 }));
    }

    fn scroll_down(&mut self) {
        if self.accounts.is_empty() { return; }
        let n = self.accounts.len();
        let i = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some((i + 1) % n));
    }
}

pub fn run(db: Database) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        crossterm::terminal::SetTitle("K1R4LABS — Riot Account Sharing"),
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(db)?;
    let res = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    res
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if handle_key(app, key.code, key.modifiers) {
                break;
            }
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode, mods: KeyModifiers) -> bool {
    if code == KeyCode::Char('c') && mods.contains(KeyModifiers::CONTROL) {
        return true;
    }

    let normal   = matches!(app.mode, Mode::Normal);
    let in_input = matches!(
        app.mode,
        Mode::SaveInput(_)
            | Mode::RenameInput(_)
            | Mode::ExportPass(_)
            | Mode::ImportPath(_)
            | Mode::ImportPass { .. }
    );
    let confirm = matches!(app.mode, Mode::ConfirmDelete | Mode::ConfirmLogout);
    let has_sel = app.selected().is_some();

    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') if normal => return true,

        KeyCode::Esc if in_input || confirm => {
            app.mode = Mode::Normal;
            app.set_ok("Cancelled");
        }

        KeyCode::Up   | KeyCode::Char('k') if normal => app.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') if normal => app.scroll_down(),

        KeyCode::Enter if normal && has_sel => app.switch_to_selected(),

        KeyCode::Char('s') | KeyCode::Char('S') if normal => {
            app.mode = Mode::SaveInput(String::new());
        }
        KeyCode::Char('r') | KeyCode::Char('R') if normal && has_sel => {
            let cur = app.selected().map(|a| a.name.clone()).unwrap_or_default();
            app.mode = Mode::RenameInput(cur);
        }
        KeyCode::Char('d') | KeyCode::Char('D') if normal && has_sel => {
            app.mode = Mode::ConfirmDelete;
        }
        KeyCode::Char('u') | KeyCode::Char('U') if normal && has_sel => {
            app.update_selected();
        }
        KeyCode::Char('e') | KeyCode::Char('E') if normal && has_sel => {
            app.mode = Mode::ExportPass(String::new());
        }
        KeyCode::Char('l') | KeyCode::Char('L') if normal => {
            app.mode = Mode::ConfirmLogout;
        }
        KeyCode::Char('i') | KeyCode::Char('I') if normal => {
            if let Some(path) = open_file_dialog() {
                app.mode = Mode::ImportPass { path, pass: String::new() };
            } else {
                app.mode = Mode::ImportPath(String::new());
            }
        }

        KeyCode::Backspace if in_input => {
            match &mut app.mode {
                Mode::SaveInput(s) | Mode::RenameInput(s) | Mode::ExportPass(s) | Mode::ImportPath(s) => {
                    s.pop();
                }
                Mode::ImportPass { pass, .. } => { pass.pop(); }
                _ => {}
            }
        }
        KeyCode::Char(c) if in_input => {
            match &mut app.mode {
                Mode::SaveInput(s) | Mode::RenameInput(s) | Mode::ExportPass(s) | Mode::ImportPath(s) => {
                    s.push(c);
                }
                Mode::ImportPass { pass, .. } => pass.push(c),
                _ => {}
            }
        }
        KeyCode::Enter if in_input => {
            let old = std::mem::replace(&mut app.mode, Mode::Normal);
            match old {
                Mode::SaveInput(text) => {
                    let name = text.trim().to_string();
                    if name.is_empty() { app.set_err("Name cannot be empty"); }
                    else { app.save_session(&name); }
                }
                Mode::RenameInput(text) => {
                    let name = text.trim().to_string();
                    if name.is_empty() { app.set_err("Name cannot be empty"); }
                    else { app.rename_selected(&name); }
                }
                Mode::ExportPass(pass) => {
                    if pass.is_empty() { app.set_err("Password cannot be empty"); }
                    else { app.export_selected(&pass); }
                }
                Mode::ImportPath(path) => {
                    let path = path.trim().to_string();
                    if path.is_empty() { app.set_err("File path cannot be empty"); }
                    else { app.mode = Mode::ImportPass { path, pass: String::new() }; }
                }
                Mode::ImportPass { path, pass } => {
                    if pass.is_empty() { app.set_err("Password cannot be empty"); }
                    else { app.import_account(&path, &pass); }
                }
                _ => {}
            }
        }

        KeyCode::Char('y') | KeyCode::Char('Y') if confirm => {
            let was = std::mem::replace(&mut app.mode, Mode::Normal);
            match was {
                Mode::ConfirmDelete  => app.delete_selected(),
                Mode::ConfirmLogout  => match riot::logout() {
                    Ok(())  => app.set_ok("Logged out — Riot Client is launching to login screen."),
                    Err(e)  => app.set_err(format!("Logout failed: {e}")),
                },
                _ => {}
            }
        }
        _ if confirm => {
            app.mode = Mode::Normal;
            app.set_ok("Cancelled");
        }

        _ => {}
    }
    false
}

fn open_file_dialog() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("Select Riot Account File")
        .add_filter("Riot Account", &["riotacc"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
}

fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    render_header(f, rows[0]);
    render_body(f, rows[1], app);
    render_footer(f, rows[2], app);

    match &app.mode {
        Mode::SaveInput(s) =>
            render_input_popup(f, "Save Current Session", "Account name:", s, false, area),
        Mode::RenameInput(s) =>
            render_input_popup(f, "Rename Account", "New name:", s, false, area),
        Mode::ExportPass(s) =>
            render_input_popup(f, "Export Account", "Set a password to protect the account:", s, true, area),
        Mode::ImportPath(s) =>
            render_input_popup(f, "Import Account  (1/2)", "File name or full path (.riotacc):", s, false, area),
        Mode::ImportPass { pass, .. } =>
            render_input_popup(f, "Import Account  (2/2)", "Password:", pass, true, area),
        Mode::ConfirmDelete => {
            let name = app.selected().map(|a| a.name.as_str()).unwrap_or("?");
            render_confirm_popup(f, "Confirm Delete", &format!("Delete \"{}\"?", name), area);
        }
        Mode::ConfirmLogout => {
            render_confirm_popup(f, "Confirm Logout", "Restart Riot Client on the login screen?", area);
        }
        Mode::Normal => {}
    }
}

fn render_header(f: &mut Frame, area: Rect) {
    let red_bold   = Style::default().fg(RED).add_modifier(Modifier::BOLD);
    let white_bold = Style::default().fg(WHITE).add_modifier(Modifier::BOLD);

    let mut lines: Vec<Line> = vec![Line::from("")];

    for i in 0..6 {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(BANNER_K1R4[i], red_bold),
            Span::raw("  "),
            Span::styled(BANNER_LABS[i], white_bold),
        ]));
    }

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("──", Style::default().fg(RED)),
        Span::raw("  "),
        Span::styled("Riot Account Sharing", Style::default().fg(MID)),
        Span::raw("  "),
        Span::styled("──", Style::default().fg(RED)),
    ]));

    f.render_widget(Paragraph::new(lines), area);
}

fn render_body(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(area);
    render_list(f, cols[0], app);
    render_detail(f, cols[1], app);
}

fn render_list(f: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app.accounts.iter().enumerate()
        .map(|(i, a)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>2}. ", i + 1), Style::default().fg(DIM)),
                Span::styled(a.name.as_str(), Style::default().fg(WHITE)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::raw(" "),
                    Span::styled("Accounts", Style::default().fg(MID)),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(DIM)),
        )
        .highlight_style(
            Style::default()
                .fg(WHITE)
                .bg(DARK)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_detail(f: &mut Frame, area: Rect, app: &App) {
    let content: Vec<Line> = if let Some(acc) = app.selected() {
        let username = riot::extract_username(&acc.token_data);
        let (riot_id_display, riot_id_style) = match acc.riot_id.clone()
            .or_else(|| riot::extract_riot_id(&acc.token_data))
        {
            Some(id) => (id, Style::default().fg(CYAN)),
            None     => ("—".to_string(), Style::default().fg(DIM)),
        };

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  *  ", Style::default().fg(RED)),
                Span::styled(acc.name.as_str(), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Riot ID   ", Style::default().fg(DIM)),
                Span::styled(riot_id_display, riot_id_style),
            ]),
        ];
        if let Some(u) = username {
            lines.push(Line::from(vec![
                Span::styled("  Login     ", Style::default().fg(DIM)),
                Span::styled(u, Style::default().fg(MID)),
            ]));
        }
        lines.extend([
            Line::from(vec![
                Span::styled("  Saved     ", Style::default().fg(DIM)),
                Span::styled(acc.saved_at.as_str(), Style::default().fg(MID)),
            ]),
            Line::from(vec![
                Span::styled("  Size      ", Style::default().fg(DIM)),
                Span::styled(format!("{} bytes", acc.token_data.len()), Style::default().fg(DIM)),
            ]),
            Line::from(""),
            Line::from(Span::styled("  ─────────────────────────────────────", Style::default().fg(DIM))),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [Enter] Switch  ", Style::default().fg(MID)),
                Span::styled("[U] Update  ", Style::default().fg(MID)),
                Span::styled("[E] Export  ", Style::default().fg(MID)),
                Span::styled("[D] Delete", Style::default().fg(MID)),
            ]),
        ]);
        lines
    } else {
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("  *  ", Style::default().fg(DIM)),
                Span::styled("No accounts saved yet.", Style::default().fg(MID)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  Log into Riot Client, then press [S] to save.",
                Style::default().fg(DIM),
            )),
        ]
    };

    f.render_widget(
        Paragraph::new(content)
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::raw(" "),
                        Span::styled("Details", Style::default().fg(MID)),
                        Span::raw(" "),
                    ]))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(DIM)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let (status_style, icon) = if app.status_err {
        (Style::default().fg(RED), "✖")
    } else {
        (Style::default().fg(GREEN), "●")
    };

    let keys =
        "  ↑↓  Nav    ↵  Switch    S  Save    U  Update    R  Rename    D  Delete    E  Export    I  Import    L  Logout    Q  Quit";

    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(format!("  {icon}  "), status_style),
                Span::styled(app.status.as_str(), status_style),
            ]),
            Line::from(Span::styled(keys, Style::default().fg(DIM))),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(DIM)),
        ),
        area,
    );
}

fn centered(w: u16, h: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w.min(area.width), h.min(area.height))
}

fn render_input_popup(f: &mut Frame, title: &str, label: &str, input: &str, masked: bool, area: Rect) {
    let popup = centered(62, 7, area);
    f.render_widget(Clear, popup);

    let display = if masked {
        format!("  {}\u{2588}", "\u{2022}".repeat(input.chars().count()))
    } else {
        format!("  {input}\u{2588}")
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(format!("  {label}"), Style::default().fg(MID))),
            Line::from(Span::styled(display, Style::default().fg(WHITE).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("  [Enter] Confirm   [Esc] Cancel", Style::default().fg(DIM))),
        ])
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(title, Style::default().fg(RED).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(RED)),
        ),
        popup,
    );
}

fn render_confirm_popup(f: &mut Frame, title: &str, message: &str, area: Rect) {
    let popup = centered(56, 6, area);
    f.render_widget(Clear, popup);

    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ⚠  ", Style::default().fg(RED)),
                Span::styled(message, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  [Y] Confirm   [Any key] Cancel",
                Style::default().fg(DIM),
            )),
        ])
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(title, Style::default().fg(RED).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(RED)),
        ),
        popup,
    );
}
