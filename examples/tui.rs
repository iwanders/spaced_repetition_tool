// Hacked up from
// https://github.com/fdehau/tui-rs/blob/v0.18.0/examples/user_input.rs

use memorizer::algorithm::memorize::recall_curve::{RecallCurveConfig, RecallCurveSelector};
use memorizer::algorithm::supermemo2::{SuperMemo2Selector};
use memorizer::recorder::YamlRecorder;
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
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

#[derive(PartialEq)]
enum ApplicationState {
    /// A question is asked, user is entering the answer.
    QuestionAsked,
    /// The true answer is displayed.
    AnswerGiven,
    /// State happens when the selector returns an empty optional. This is a termination state.
    NoMoreQuestions,
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
        let learnables = load_text_learnables(
            &std::env::args()
                .nth(1)
                .expect("Provide argument to learnables.yaml"),
        )?;
        let recorder = YamlRecorder::new("../log.yaml")?;

        // let config: RecallCurveConfig = Default::default();
        // let selector = RecallCurveSelector::new(config);
        let selector = SuperMemo2Selector::new();
        
        let training = Training::new(learnables, Box::new(recorder), Box::new(selector));
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
        if let Some(q) = self.training.question() {
            self.question = q;
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
        } else {
            self.original.clear();
            self.transform = String::from("No more questions at the moment.");
            self.input.clear();
            self.state = ApplicationState::NoMoreQuestions;
        }
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
            if key.code == KeyCode::Esc {
                return Ok(());
            }

            match app.state {
                ApplicationState::QuestionAsked => match key.code {
                    KeyCode::Enter => {
                        app.process_answer();
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
                    _ => {}
                },
                ApplicationState::AnswerGiven => match key.code {
                    KeyCode::Enter => {
                        app.populate_new();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let vertical_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());
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
        .split(vertical_split[1]);

    const FROM: usize = 1;
    const TRANSFORM: usize = 3;
    const INPUT: usize = 5;
    const ANSWER: usize = 6;

    let msg = vec![
        Span::raw("Press "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to exit, "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to submit answer."),
    ];

    let style = Style::default();
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, vertical_split[0]);

    let orig = Paragraph::new(app.original.as_ref())
        // .alignment(tui::layout::Alignment::Center)
        .block(Block::default());
    f.render_widget(orig, chunks[FROM]);

    let transform = Paragraph::new(app.transform.as_ref())
        // .alignment(tui::layout::Alignment::Center)
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
        ApplicationState::NoMoreQuestions => {
            input_style = Style::default();
        }
    }

    let input = Paragraph::new(app.input.as_ref())
        .style(input_style)
        // .alignment(tui::layout::Alignment::Center)
        .block(Block::default());
    f.render_widget(input, chunks[INPUT]);

    if !app.answer_correct {
        let answer = Paragraph::new(app.answer.as_ref())
            // .alignment(tui::layout::Alignment::Center)
            .block(Block::default());
        f.render_widget(answer, chunks[ANSWER]);
    }
}
