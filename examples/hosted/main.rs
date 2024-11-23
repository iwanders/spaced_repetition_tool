use memorizer::recorder::YamlRecorder;
use memorizer::text::load_text_learnables; // TextRepresentation
use memorizer::training::Training;
// use memorizer::traits::{Learnable, Question, Record, Recorder, RepresentationId, Score, Selector};
use memorizer::traits::{LearnableId, Question, Record, Selector};

use std::sync::Arc;
use std::thread;

use ascii::AsciiString;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

fn get_content_type(path: &Path) -> &'static str {
    let extension = match path.extension() {
        None => return "text/plain",
        Some(e) => e,
    };

    match extension.to_str().unwrap() {
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "gif" => "image/gif",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        _ => "text/plain; charset=utf8",
    }
}

fn file_to_response(path: &std::path::Path, file: std::fs::File) -> Response<std::fs::File> {
    let response = tiny_http::Response::from_file(file);
    response.with_header(tiny_http::Header {
        field: "Content-Type".parse().unwrap(),
        value: AsciiString::from_ascii(get_content_type(path)).unwrap(),
    })
}

#[derive(Deserialize, Serialize, Debug)]
enum SelectorOptions {
    SuperMemo2,
    RecallCurveSelector,
}
impl SelectorOptions {
    pub fn make_selector(&self) -> Box<dyn Selector> {
        match *self {
            SelectorOptions::SuperMemo2 => {
                Box::new(memorizer::algorithm::super_memo_2::SuperMemo2Selector::new())
            }
            SelectorOptions::RecallCurveSelector => {
                use memorizer::algorithm::memorize::recall_curve::RecallCurveSelector;
                Box::new(RecallCurveSelector::new(Default::default()))
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, Hash, PartialEq)]
#[serde(transparent)]
struct DeckName(pub String);

#[derive(Deserialize, Serialize, Debug, Clone, Eq, Hash, PartialEq)]
#[serde(transparent)]
struct UserName(pub String);

#[derive(Deserialize, Serialize, Debug)]
struct NamedDeck {
    name: DeckName,
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct UserDecks {
    username: UserName,
    decks: Vec<NamedDeck>,
}

#[derive(Deserialize, Serialize, Debug)]
struct HostConfig {
    selector: SelectorOptions,
    user_decks: Vec<UserDecks>,
}

use parking_lot::RwLock;

type ThreadsafeTraining = RwLock<Training>;

type UserTraining = std::collections::HashMap<DeckName, ThreadsafeTraining>;

#[derive(Default)]
struct TrainingBackend {
    entries: std::collections::HashMap<UserName, UserTraining>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FullTextQuestion {
    from: String,
    transform: String,
    to: String,
    learnable: LearnableId,
}

impl TrainingBackend {
    pub fn users(&self) -> Vec<UserName> {
        self.entries.keys().cloned().collect()
    }
    pub fn decks(&self, user: &UserName) -> Vec<DeckName> {
        if let Some(user_decks) = self.entries.get(user) {
            user_decks.keys().into_iter().cloned().collect()
        } else {
            vec![]
        }
    }

    pub fn question(
        &self,
        user: &UserName,
        deck: &DeckName,
    ) -> Result<Option<FullTextQuestion>, BackendError> {
        let users_decks = self.entries.get(user).ok_or(format!("no user {user:?}"))?;
        let deck = users_decks.get(deck).ok_or(format!("no user {deck:?}"))?;
        let mut deck = deck.write();
        if let Some(v) = deck.question() {
            let answer_repr = deck.get_answer(&v)?;
            let from_repr = deck.representation(v.from);
            let transform = deck.transform(v.transform);
            Ok(Some(FullTextQuestion {
                from: from_repr.text().to_owned(),
                transform: transform.description().to_owned(),
                to: answer_repr.text().to_owned(),
                learnable: v.learnable,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn rate_question(
        &self,
        user: &UserName,
        deck_name: &DeckName,
        learnable: LearnableId,
        score: f64,
    ) -> Result<(), BackendError> {
        let users_decks = self.entries.get(user).ok_or(format!("no user {user:?}"))?;
        let deck = users_decks
            .get(deck_name)
            .ok_or(format!("no user {deck_name:?}"))?;
        let mut deck = deck.write();
        if let Some(question) = deck.question_from_learnable(learnable) {
            let record = Record {
                question: question,
                score: score.clamp(0.0, 1.0),
                time: std::time::SystemTime::now(),
            };
            deck.finalize_answer(record)?;
            Ok(())
        } else {
            Err(format!("could not find {learnable:?} in deck {deck_name:?} for {user:?}").into())
        }
    }
}

impl TrainingBackend {
    pub fn from_config(config: &HostConfig, storage_dir: &str) -> Result<Self, BackendError> {
        let mut res = TrainingBackend::default();
        let storage_dir = PathBuf::from(storage_dir);
        for user_deck in config.user_decks.iter() {
            let user_map = res.entries.entry(user_deck.username.clone()).or_default();
            for deck in user_deck.decks.iter() {
                let selector = config.selector.make_selector();
                // Load the actual deck.
                let deck_learnables = load_text_learnables(&deck.path)?;

                let recorder_file_path = storage_dir
                    .join(&user_deck.username.0)
                    .join(format!("{}_recording.yaml", deck.name.0));
                let recorder = YamlRecorder::new(&recorder_file_path)?;
                let training = Training::new(deck_learnables, Box::new(recorder), selector);
                user_map.insert(deck.name.clone(), training.into());
            }
        }
        Ok(res)
    }
}

use std::path::PathBuf;
use tiny_http::Request;
use tiny_http::Response;
use tiny_http::ResponseBox;

type BackendError = Box<dyn std::error::Error + Send + Sync>;
struct Hoster {
    frontend_root: PathBuf,
    backend: TrainingBackend,
}

impl Hoster {
    pub fn new(frontend_root: &str, backend: TrainingBackend) -> Result<Self, BackendError> {
        let frontend_root = PathBuf::from(frontend_root);
        // let frontend_root = frontend_path.join("www");
        if !frontend_root.is_dir() {
            return Err("frontend path not to directory".into());
        }
        Ok(Hoster {
            frontend_root,
            backend,
        })
    }

    fn serve_file(&self, path: &Path) -> Result<Option<ResponseBox>, BackendError> {
        let path = self.frontend_root.join(path);
        println!("Path: {path:?}");
        if !path.is_file() {
            return Ok(None);
        }
        let file = fs::File::open(&path);

        if file.is_ok() {
            return Ok(Some(file_to_response(&path, file.unwrap()).boxed()));
        } else {
            return Err("could not open file".into());
        }
    }

    pub fn request_file(&self, rq: &Request) -> Result<Option<ResponseBox>, BackendError> {
        let url = rq.url().to_string();
        let path = url.strip_prefix("/").unwrap();
        let path = if path == "" { "index.html" } else { path };
        self.serve_file(Path::new(&path))
    }

    pub fn backend_api(&self, rq: &mut Request) -> Result<Option<ResponseBox>, BackendError> {
        let url = rq.url().to_string();
        let path = url.strip_prefix("/").unwrap();
        match path {
            "api/test" => {
                #[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq, Serialize, Deserialize)]
                struct TestRequest {
                    value: usize,
                }

                let mut content = String::new();
                rq.as_reader().read_to_string(&mut content)?;
                let _r: TestRequest = serde_json::from_str(&content)?;
                Ok(Some(
                    tiny_http::Response::from_string("{\"response\":\"queued\"}")
                        .with_status_code(tiny_http::StatusCode(200))
                        .boxed(),
                ))
            }
            "api/users" => {
                #[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq, Serialize, Deserialize)]
                struct UserResponse {
                    users: Vec<String>,
                }
                let resp = UserResponse {
                    users: self
                        .backend
                        .users()
                        .into_iter()
                        .map(|z| z.0.clone())
                        .collect(),
                };

                Ok(Some(
                    tiny_http::Response::from_string(serde_json::to_string_pretty(&resp).unwrap())
                        .with_status_code(tiny_http::StatusCode(200))
                        .boxed(),
                ))
            }
            full_path if path.starts_with("api/deck/") => {
                let user = full_path.replace("api/deck/", "");
                #[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq, Serialize, Deserialize)]
                struct DeckResponse {
                    decks: Vec<String>,
                }
                let user = UserName(user.to_owned());
                let resp = DeckResponse {
                    decks: self
                        .backend
                        .decks(&user)
                        .into_iter()
                        .map(|z| z.0.clone())
                        .collect(),
                };

                Ok(Some(
                    tiny_http::Response::from_string(serde_json::to_string_pretty(&resp).unwrap())
                        .with_status_code(tiny_http::StatusCode(200))
                        .boxed(),
                ))
            }
            full_path if path.starts_with("api/question/") => {
                let query = full_path.replace("api/question/", "");
                let mut elements = query.split("/");
                let user = elements.next().ok_or(format!("no user provided"))?;
                let deck = elements.next().ok_or(format!("no deck provided"))?;

                let user = UserName(user.to_owned());
                let deck = DeckName(deck.to_owned());

                let resp = self.backend.question(&user, &deck)?;
                Ok(Some(
                    tiny_http::Response::from_string(serde_json::to_string_pretty(&resp).unwrap())
                        .with_status_code(tiny_http::StatusCode(200))
                        .boxed(),
                ))
            }
            full_path if path.starts_with("api/submit_answer/") => {
                let query = full_path.replace("api/submit_answer/", "");
                let mut elements = query.split("/");
                let user = elements.next().ok_or(format!("no user provided"))?;
                let deck = elements.next().ok_or(format!("no deck provided"))?;

                let user = UserName(user.to_owned());
                let deck = DeckName(deck.to_owned());

                #[derive(Debug, Clone, Serialize, Deserialize)]
                struct SubmitRating {
                    score: f64,
                    question: FullTextQuestion,
                }

                let mut content = String::new();
                rq.as_reader().read_to_string(&mut content)?;
                let submit_rating: SubmitRating = serde_json::from_str(&content)?;

                // We got a question from the user, verify that this actually exists.
                println!("req: {submit_rating:?}");
                self.backend.rate_question(
                    &user,
                    &deck,
                    submit_rating.question.learnable,
                    submit_rating.score,
                )?;
                Ok(Some(
                    tiny_http::Response::from_string("{\"response\":\"stored\"}")
                        .with_status_code(tiny_http::StatusCode(200))
                        .boxed(),
                ))
            }
            _ if path.starts_with("?") => {
                // handle anything starting with ? by the index.html, this allows it to behave as a
                // single page application while still taking state from the url.
                return self.serve_file(&std::path::PathBuf::from("index.html"));
            }
            _ => Ok(None),
        }
    }
}

use clap::Parser;

/// A hosted training session
#[derive(Parser, Debug)]
#[clap(long_about = None)]
struct Args {
    /// The input configuration file.
    config: String,

    /// Directory to the frontend www directory, defaults to ./examples/hosted/www to work with `cargo r`
    #[clap(short, long, default_value = "./examples/hosted/www/")]
    www: String,

    /// Storage directory
    #[clap(short, long, default_value = "/tmp/")]
    storage: String,
}

pub fn main() -> Result<(), BackendError> {
    let v = tiny_http::Server::http("0.0.0.0:8080")?;
    let server = Arc::new(v);
    let port = server
        .server_addr()
        .to_ip()
        .expect("only using ip sockets")
        .port();
    println!("Now listening on port {}", port);

    let args = Args::parse();

    let file = std::fs::File::open(args.config)?;
    let config: HostConfig = serde_yaml::from_reader(file)?;

    let training_backend = TrainingBackend::from_config(&config, &args.storage)?;

    let backend = Arc::new(Hoster::new(&args.www, training_backend)?);

    // Serve the webserver with 4 threads.
    let mut handles = Vec::new();
    for _ in 0..4 {
        let server = server.clone();
        let backend = backend.clone();

        handles.push(thread::spawn(move || {
            for mut rq in server.incoming_requests() {
                // println!("{:?}", rq);

                type Handler<'a> =
                    &'a dyn Fn(&mut Request) -> Result<Option<ResponseBox>, BackendError>;

                let order: [Handler; 2] = [
                    // Files take precedence
                    &|r| backend.request_file(r),
                    // over api endpoints
                    &|r| backend.backend_api(r),
                ];

                let url = rq.url().to_string();

                let mut r = vec![];
                for t in order {
                    let z = t(&mut rq);
                    let served = if z.is_ok() {
                        z.as_ref().ok().unwrap().is_some() == true
                    } else {
                        false
                    };

                    r.push(z);
                    if served {
                        break;
                    }
                }

                if !r
                    .iter()
                    .map(|v| {
                        if v.is_ok() {
                            v.as_ref().ok().unwrap().is_some()
                        } else {
                            true // error, it was definitely handled, but something went bad.
                        }
                    })
                    .any(|v| v)
                {
                    let rep = tiny_http::Response::from_string("Nothing handles this request")
                        .with_status_code(tiny_http::StatusCode(500));
                    let _ = rq.respond(rep);
                    println!("Nothing handled this request: {url:?}");
                    continue;
                }

                for z in r {
                    match z {
                        Err(e) => {
                            let rep = tiny_http::Response::from_string(format!("{:?}", e))
                                .with_status_code(tiny_http::StatusCode(500));
                            println!("Error {url:?}-> {e:?}");
                            let _ = rq.respond(rep);

                            break;
                        }
                        Ok(v) => {
                            if let Some(v) = v {
                                let _ = rq.respond(v);
                                break;
                            }
                        }
                    }
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
    Ok(())
}
