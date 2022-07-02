// Hacked up from
// https://github.com/fdehau/tui-rs/blob/v0.18.0/examples/user_input.rs

// This is a pretty clunky text interface, but hey it works.

// use memorizer::algorithm::memorize::recall_curve::{RecallCurveConfig, RecallCurveSelector};
use memorizer::algorithm::super_memo_2::SuperMemo2Selector;
use memorizer::recorder::YamlRecorder;
use memorizer::text::{load_text_learnables, TextRepresentation};
use memorizer::training::Training;
use memorizer::traits::{Question, Record, RepresentationId, Score};

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
    widgets::{Block, Paragraph}, // Borders,
    Frame,
    Terminal,
};

#[derive(PartialEq)]
enum ApplicationState {
    /// A question is asked, user is entering the answer.
    QuestionAsked,
    /// The true answer is displayed, score can be adjusted. Pressing enter submits this record.
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

    /// The proposed record for this answer.
    record: Option<Record>,
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
            record: Default::default(),
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
        let (record, truth) = self
            .training
            .get_answer(&self.question, z)
            .expect("should succeed");

        self.answer_score = record.score;
        self.answer_correct = record.score == 1.0;

        self.record = Some(record);

        self.answer = truth.text().to_string();
        self.state = ApplicationState::AnswerGiven;
    }

    fn submit_record(&mut self) {
        if let Some(record) = self.record {
            self.training
                .finalize_answer(record)
                .expect("Shouldn't fail");
        }
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

    fn modify_pending_score(&mut self, v: f64) {
        let record = self
            .record
            .as_mut()
            .expect("Must be set populated when modifying");
        record.score = (record.score + v).clamp(0.0, 1.0);
        record.score = (record.score * 10.0).round() / 10.0;
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
                        if !app.input.is_empty() {
                            app.process_answer();
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
                    _ => {}
                },
                ApplicationState::AnswerGiven => match key.code {
                    KeyCode::Enter => {
                        app.submit_record();
                        app.populate_new();
                    }
                    KeyCode::Right | KeyCode::Up => {
                        // println!("adding");
                        app.modify_pending_score(0.2);
                    }
                    KeyCode::Left | KeyCode::Down => {
                        app.modify_pending_score(-0.2);
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
                Constraint::Length(1), // Scorebar
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(vertical_split[1]);

    const FROM: usize = 1;
    const TRANSFORM: usize = 3;
    const INPUT: usize = 5;
    const ANSWER: usize = 6;
    const SCOREBAR: usize = 7;
    let mut score_bar_region = chunks[SCOREBAR];
    score_bar_region.width = 3 * 5;

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

    // .alignment(tui::layout::Alignment::Center)
    let orig = Paragraph::new(app.original.as_ref()).block(Block::default());
    f.render_widget(orig, chunks[FROM]);

    let transform = Paragraph::new(app.transform.as_ref()).block(Block::default());
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
            let score = app
                .record
                .as_ref()
                .expect("Must be populated in answer given state")
                .score;
            let fg_color;
            let label;

            if score == 0.0 {
                fg_color = Color::Red;
                label = "blackout";
            } else if score == 0.2 {
                fg_color = Color::Red;
                label = "familiar";
            } else if score == 0.4 {
                fg_color = Color::Red;
                label = "ah yes";
            } else if score == 0.6 {
                fg_color = Color::Green;
                label = "effort";
            } else if score == 0.8 {
                fg_color = Color::Green;
                label = "hesitated";
            } else {
                fg_color = Color::Green;
                label = "aced";
            }
            let g = tui::widgets::Gauge::default()
                .block(Block::default())
                .gauge_style(
                    Style::default()
                        .fg(fg_color)
                        .bg(Color::Black)
                        .add_modifier(Modifier::ITALIC),
                )
                .ratio(score)
                .label(label);
            f.render_widget(g, score_bar_region);
        }
        ApplicationState::NoMoreQuestions => {
            input_style = Style::default();
        }
    }

    let input = Paragraph::new(app.input.as_ref())
        .style(input_style)
        .block(Block::default());
    f.render_widget(input, chunks[INPUT]);

    if !app.answer_correct {
        let answer = Paragraph::new(app.answer.as_ref()).block(Block::default());
        f.render_widget(answer, chunks[ANSWER]);
    }
}
