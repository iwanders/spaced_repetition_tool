// Hacked up from
// https://github.com/fdehau/tui-rs/blob/v0.18.0/examples/user_input.rs

use memorizer::recorder::MemoryRecorder;
use memorizer::text::load_text_learnables;
use memorizer::training::Training;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

/// App holds the state of the application
struct App {
    /// Source representation
    original: String,

    /// Transformation to perform.
    transform: String,

    /// Current value of the input box
    input: String,

    /// Object that holds the training loop.
    training: Training,
}

impl App {
    fn new() -> Result<App, Box<dyn Error>> {
        let learnables = load_text_learnables("/tmp/output.yaml")?;
        let recorder = MemoryRecorder::new();
        let training = Training::new(learnables, Box::new(recorder));
        Ok(App {
            input: String::new(),
            training,
            original: String::new(),
            transform: String::new(),
        })
    }

    fn process_answer(&mut self) {
        // do something with the current input.
        self.populate_new();
    }

    fn populate_new(&mut self) {
        let new_q = self.training.question();
        self.original = new_q.0.get_text().to_string();
        self.transform = new_q.1.get_description().to_string();
        self.input.clear();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new()?;
    app.populate_new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    //app.messages.push(app.input.drain(..).collect());
                    app.process_answer();
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Length(1),
                Constraint::Percentage(5),
                Constraint::Length(1),
                Constraint::Percentage(5),
                Constraint::Length(1),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(f.size());

    let msg = vec![
        Span::raw("Press "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to exit, "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to submit asnwer."),
    ];
    let style = Style::default();
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let orig = Paragraph::new(app.original.as_ref())
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default());
    f.render_widget(orig, chunks[1]);

    let transform = Paragraph::new(app.transform.as_ref())
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default());
    f.render_widget(transform, chunks[3]);

    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default());
    f.render_widget(input, chunks[5]);
}
