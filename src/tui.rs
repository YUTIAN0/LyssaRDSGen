//! Terminal User Interface

use crate::keygen::{generate_lkp, generate_spk, validate_tskey};
use crate::types::{LicenseInfo, SPKCurve, LICENSE_TYPES};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use num_bigint::BigUint;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

enum InputField {
    Pid,
    Spk,
    Count,
    License,
}

enum FocusedWidget {
    Input(InputField),
    GenerateSpk,
    ValidateSpk,
    GenerateLkp,
}

pub struct TuiApp {
    pid: String,
    spk: String,
    count: String,
    license_state: ListState,
    generated_spk: String,
    generated_lkp: String,
    status_message: String,
    focused: FocusedWidget,
    should_quit: bool,
}

impl TuiApp {
    fn new() -> Self {
        let mut license_state = ListState::default();
        license_state.select(Some(18)); // Default to Windows Server 2022 Per Device
        
        Self {
            pid: String::new(),
            spk: String::new(),
            count: String::from("1"),
            license_state,
            generated_spk: String::new(),
            generated_lkp: String::new(),
            status_message: String::new(),
            focused: FocusedWidget::Input(InputField::Pid),
            should_quit: false,
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.next_field();
            }
            KeyCode::BackTab => {
                self.prev_field();
            }
            KeyCode::Enter => {
                self.handle_enter();
            }
            KeyCode::Char(c) => {
                self.handle_char(c);
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            KeyCode::Up => {
                if matches!(self.focused, FocusedWidget::Input(InputField::License)) {
                    self.prev_license();
                }
            }
            KeyCode::Down => {
                if matches!(self.focused, FocusedWidget::Input(InputField::License)) {
                    self.next_license();
                }
            }
            _ => {}
        }
    }

    fn next_field(&mut self) {
        self.focused = match self.focused {
            FocusedWidget::Input(InputField::Pid) => FocusedWidget::Input(InputField::Spk),
            FocusedWidget::Input(InputField::Spk) => FocusedWidget::Input(InputField::Count),
            FocusedWidget::Input(InputField::Count) => FocusedWidget::Input(InputField::License),
            FocusedWidget::Input(InputField::License) => FocusedWidget::GenerateSpk,
            FocusedWidget::GenerateSpk => FocusedWidget::ValidateSpk,
            FocusedWidget::ValidateSpk => FocusedWidget::GenerateLkp,
            FocusedWidget::GenerateLkp => FocusedWidget::Input(InputField::Pid),
        };
    }

    fn prev_field(&mut self) {
        self.focused = match self.focused {
            FocusedWidget::Input(InputField::Pid) => FocusedWidget::GenerateLkp,
            FocusedWidget::Input(InputField::Spk) => FocusedWidget::Input(InputField::Pid),
            FocusedWidget::Input(InputField::Count) => FocusedWidget::Input(InputField::Spk),
            FocusedWidget::Input(InputField::License) => FocusedWidget::Input(InputField::Count),
            FocusedWidget::GenerateSpk => FocusedWidget::Input(InputField::License),
            FocusedWidget::ValidateSpk => FocusedWidget::GenerateSpk,
            FocusedWidget::GenerateLkp => FocusedWidget::ValidateSpk,
        };
    }

    fn handle_char(&mut self, c: char) {
        match &self.focused {
            FocusedWidget::Input(InputField::Pid) => self.pid.push(c),
            FocusedWidget::Input(InputField::Spk) => self.spk.push(c),
            FocusedWidget::Input(InputField::Count) => {
                if c.is_ascii_digit() {
                    self.count.push(c);
                }
            }
            _ => {}
        }
    }

    fn handle_backspace(&mut self) {
        match &self.focused {
            FocusedWidget::Input(InputField::Pid) => {
                self.pid.pop();
            }
            FocusedWidget::Input(InputField::Spk) => {
                self.spk.pop();
            }
            FocusedWidget::Input(InputField::Count) => {
                self.count.pop();
            }
            _ => {}
        }
    }

    fn next_license(&mut self) {
        let i = match self.license_state.selected() {
            Some(i) => {
                if i >= LICENSE_TYPES.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.license_state.select(Some(i));
    }

    fn prev_license(&mut self) {
        let i = match self.license_state.selected() {
            Some(i) => {
                if i == 0 {
                    LICENSE_TYPES.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.license_state.select(Some(i));
    }

    fn handle_enter(&mut self) {
        match self.focused {
            FocusedWidget::GenerateSpk => self.generate_spk(),
            FocusedWidget::ValidateSpk => self.validate_spk(),
            FocusedWidget::GenerateLkp => self.generate_lkp(),
            _ => {}
        }
    }

    fn generate_spk(&mut self) {
        if self.pid.trim().is_empty() {
            self.status_message = "Error: PID is required".to_string();
            return;
        }

        match generate_spk(&self.pid) {
            Ok(spk) => {
                self.generated_spk = spk;
                self.status_message = "SPK generated successfully!".to_string();
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    fn validate_spk(&mut self) {
        if self.pid.trim().is_empty() {
            self.status_message = "Error: PID is required".to_string();
            return;
        }

        if self.spk.trim().is_empty() {
            self.status_message = "Error: SPK is required for validation".to_string();
            return;
        }

        match validate_tskey(
            &self.pid,
            &self.spk,
            SPKCurve::gx(),
            SPKCurve::gy(),
            SPKCurve::kx(),
            SPKCurve::ky(),
            BigUint::from(SPKCurve::A),
            SPKCurve::p(),
            true,
        ) {
            Ok(true) => {
                self.status_message = "SPK validation successful!".to_string();
            }
            Ok(false) => {
                self.status_message = "Error: SPK does not match the PID".to_string();
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    fn generate_lkp(&mut self) {
        if self.pid.trim().is_empty() {
            self.status_message = "Error: PID is required".to_string();
            return;
        }

        let count: u32 = match self.count.parse() {
            Ok(c) if (1..=9999).contains(&c) => c,
            _ => {
                self.status_message = "Error: Count must be between 1 and 9999".to_string();
                return;
            }
        };

        let selected = self.license_state.selected().unwrap_or(0);
        let license_type = LICENSE_TYPES[selected].0;
        
        let license_info = match LicenseInfo::parse(license_type) {
            Ok(info) => info,
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                return;
            }
        };

        match generate_lkp(
            &self.pid,
            count,
            license_info.chid,
            license_info.major_ver,
            license_info.minor_ver,
        ) {
            Ok(lkp) => {
                self.generated_lkp = lkp;
                self.status_message = format!(
                    "LKP generated successfully! ({})",
                    license_info.description
                );
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status
            Constraint::Length(2),  // Help
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("LyssaRDSGen - RDS License Key Generator")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left panel - Inputs
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // PID
            Constraint::Length(3),  // SPK
            Constraint::Length(3),  // Count
            Constraint::Min(5),     // License
            Constraint::Length(3),  // Buttons
        ])
        .split(main_chunks[0]);

    // PID input
    let pid_style = if matches!(app.focused, FocusedWidget::Input(InputField::Pid)) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let pid_input = Paragraph::new(app.pid.as_str())
        .block(Block::default().borders(Borders::ALL).title("Product ID").border_style(pid_style));
    f.render_widget(pid_input, left_chunks[0]);

    // SPK input
    let spk_style = if matches!(app.focused, FocusedWidget::Input(InputField::Spk)) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let spk_input = Paragraph::new(app.spk.as_str())
        .block(Block::default().borders(Borders::ALL).title("Existing SPK (Optional)").border_style(spk_style));
    f.render_widget(spk_input, left_chunks[1]);

    // Count input
    let count_style = if matches!(app.focused, FocusedWidget::Input(InputField::Count)) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let count_input = Paragraph::new(app.count.as_str())
        .block(Block::default().borders(Borders::ALL).title("License Count (1-9999)").border_style(count_style));
    f.render_widget(count_input, left_chunks[2]);

    // License type list
    let license_style = if matches!(app.focused, FocusedWidget::Input(InputField::License)) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let licenses: Vec<ListItem> = LICENSE_TYPES
        .iter()
        .map(|(_, desc)| ListItem::new(*desc))
        .collect();
    let licenses_list = List::new(licenses)
        .block(Block::default().borders(Borders::ALL).title("License Type (↑↓ to select)").border_style(license_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(licenses_list, left_chunks[3], &mut app.license_state);

    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(left_chunks[4]);

    let gen_spk_style = if matches!(app.focused, FocusedWidget::GenerateSpk) {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else {
        Style::default().fg(Color::Green)
    };
    let gen_spk_btn = Paragraph::new("Generate SPK")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(gen_spk_style));
    f.render_widget(gen_spk_btn, button_chunks[0]);

    let val_spk_style = if matches!(app.focused, FocusedWidget::ValidateSpk) {
        Style::default().fg(Color::Black).bg(Color::Blue)
    } else {
        Style::default().fg(Color::Blue)
    };
    let val_spk_btn = Paragraph::new("Validate SPK")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(val_spk_style));
    f.render_widget(val_spk_btn, button_chunks[1]);

    let gen_lkp_style = if matches!(app.focused, FocusedWidget::GenerateLkp) {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let gen_lkp_btn = Paragraph::new("Generate LKP")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(gen_lkp_style));
    f.render_widget(gen_lkp_btn, button_chunks[2]);

    // Right panel - Output
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    // SPK output
    let spk_output = Paragraph::new(app.generated_spk.as_str())
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Generated SPK"))
        .wrap(Wrap { trim: false });
    f.render_widget(spk_output, right_chunks[0]);

    // LKP output
    let lkp_output = Paragraph::new(app.generated_lkp.as_str())
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("Generated LKP"))
        .wrap(Wrap { trim: false });
    f.render_widget(lkp_output, right_chunks[1]);

    // Status bar
    let status_color = if app.status_message.starts_with("Error") {
        Color::Red
    } else {
        Color::Green
    };
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(status_color))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[2]);

    // Help bar
    let help_text = "Tab: Next field | Shift+Tab: Prev | Enter: Execute | ↑↓: Select license | Esc/q: Quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[3]);
}

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = TuiApp::new();

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
