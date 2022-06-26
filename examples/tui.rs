// Hacked up from
// https://github.com/fdehau/tui-rs/blob/v0.18.0/examples/user_input.rs

use memorizer::recorder::{MemoryRecorder, YamlRecorder};
use memorizer::text::{load_text_learnables, TextRepresentation};
use memorizer::training::Training;
use memorizer::traits::{Question, RepresentationId, Score};

use std::rc::Rc;

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

#[derive(PartialEq)]
enum ApplicationState {
    QuestionAsked,
    AnswerGiven,
}

/// App holds the state of the application
struct App {
    state: ApplicationState,

    /// Source representation
    original: String,

    /// Transformation to perform.
    transform: String,

    /// Current value of the input box
    input: String,

    /// Object that holds the training loop.
    training: Training,

    /// String holding the real answer.
    answer: String,

    /// The answer score.
    answer_score: Score,

    /// Answer correct?
    answer_correct: bool,

    /// The current question:
    question: Question,
}

impl App {
    fn new() -> Result<App, Box<dyn Error>> {
        let learnables = load_text_learnables("/tmp/output.yaml")?;
        let recorder = YamlRecorder::new("/tmp/log.yaml")?;
        let training = Training::new(learnables, Box::new(recorder));
        Ok(App {
            input: String::new(),
            training,
            original: String::new(),
            transform: String::new(),
            answer: String::new(),
            answer_score: 0.0,
            answer_correct: false,
            state: ApplicationState::QuestionAsked,
            question: Default::default(),
        })
    }

    fn clear_fields(&mut self) {
        self.input.clear();
        self.original.clear();
        self.transform.clear();
        self.answer.clear();
    }

    fn process_answer(&mut self) {
        // do something with the current input.
        let z = Rc::new(TextRepresentation::new(&self.input, RepresentationId(0)));
        let (score, truth) = self
            .training
            .answer(&self.question, z)
            .expect("should succeed");
        self.answer_score = score;
        self.answer_correct = score == 1.0;
        self.answer = truth.text().to_string();
        self.state = ApplicationState::AnswerGiven;
    }

    fn populate_new(&mut self) {
        self.clear_fields();
        self.question = self.training.question();
        self.original = self
            .training
            .representation(self.question.from)
            .text()
            .to_string();
        self.transform = self
            .training
            .transform(self.question.transform)
            .description()
            .to_string();
        self.input.clear();
        self.state = ApplicationState::QuestionAsked;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // create app and run it
    let mut app = App::new()?;
    app.populate_new();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Now run the application.
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
                    match app.state {
                        ApplicationState::QuestionAsked => {
                            app.process_answer();
                        }
                        ApplicationState::AnswerGiven => {
                            app.populate_new();
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if app.state == ApplicationState::QuestionAsked {
                        app.input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if app.state == ApplicationState::QuestionAsked {
                        app.input.pop();
                    }
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
                Constraint::Length(1), // from
                Constraint::Percentage(5),
                Constraint::Length(1), // transform
                Constraint::Percentage(5),
                Constraint::Length(1), // input
                Constraint::Length(1), // real answer
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(f.size());

    const FROM: usize = 1;
    const TRANSFORM: usize = 3;
    const INPUT: usize = 5;
    const ANSWER: usize = 6;

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
    f.render_widget(orig, chunks[FROM]);

    let transform = Paragraph::new(app.transform.as_ref())
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default());
    f.render_widget(transform, chunks[TRANSFORM]);

    let input_style;
    match app.state {
        ApplicationState::QuestionAsked => {
            input_style = Style::default().fg(Color::Yellow);
        }
        ApplicationState::AnswerGiven => {
            if app.answer_correct {
                input_style = Style::default().fg(Color::Green);
            } else {
                input_style = Style::default().fg(Color::Red);
            }
        }
    }

    let input = Paragraph::new(app.input.as_ref())
        .style(input_style)
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default());
    f.render_widget(input, chunks[INPUT]);

    if !app.answer_correct {
        let answer = Paragraph::new(app.answer.as_ref())
            .alignment(tui::layout::Alignment::Center)
            .block(Block::default());
        f.render_widget(answer, chunks[ANSWER]);
    }
}
