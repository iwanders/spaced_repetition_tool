// use memorizer::recorder::YamlRecorder;
// use memorizer::text::{load_text_learnables, TextRepresentation};
use memorizer::training::Training;
// use memorizer::traits::{Question, Record, RepresentationId, Score, Selector};

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


use parking_lot::RwLock;
struct TrainingInterface {
    training: RwLock<Training>,
}
impl TrainingInterface {

}

use std::path::PathBuf;
use tiny_http::Request;
use tiny_http::Response;
use tiny_http::ResponseBox;

type BackendError = Box<dyn std::error::Error + Send + Sync>;
struct Backend {
    frontend_root: PathBuf,
}

impl Backend {
    pub fn new(
        frontend_root: &std::path::Path,
    ) -> Result<Self, BackendError> {
        // let frontend_root = frontend_path.join("www");
        if !frontend_root.is_dir() {
            return Err("frontend path not to directory".into());
        }
        Ok(Backend {
            frontend_root: frontend_root.to_path_buf(),
        })
    }

    pub fn request_file(&self, rq: &Request) -> Result<Option<ResponseBox>, BackendError> {
        let url = rq.url().to_string();
        let path = url.strip_prefix("/").unwrap();
        let path = if path == "" {
            "index.html"
        } else {
            path
        };
        let path = self.frontend_root.join(Path::new(&path));
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

    pub fn backend_api(&self, rq: &mut Request) -> Result<Option<ResponseBox>, BackendError> {
        let url = rq.url().to_string();
        let path = url.strip_prefix("/").unwrap();
        match path {
            "test" => {
                #[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq, Serialize, Deserialize)]
                struct TestRequest {
                    value: usize,
                }

                let mut content = String::new();
                rq.as_reader().read_to_string(&mut content)?;
                let r: TestRequest = serde_json::from_str(&content)?;
                Ok(Some(
                    tiny_http::Response::from_string("{\"response\":\"queued\"}")
                        .with_status_code(tiny_http::StatusCode(200))
                        .boxed(),
                ))
            }
            _ => Ok(None),
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let v = tiny_http::Server::http("0.0.0.0:8080")?;
    let server = Arc::new(v);
    let port = server.server_addr().to_ip().expect("only using ip sockets").port();
    println!("Now listening on port {}", port);

    let backend = Arc::new(Backend::new(
        &std::env::current_dir()?.join("examples/www/"),
    )?);

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

                let order: [Handler; 2] =
                    [
                        // Files take precedence
                        &|r| backend.request_file(r),
                        // over api endpoints
                        &|r| backend.backend_api(r)
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
