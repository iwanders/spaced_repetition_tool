use memorizer::text::{save_text_learnables, TextLearnable, TextRepresentation, TextTransform};
use memorizer::traits::{LearnableId, RepresentationId, TransformId};

use clap::Parser;

/// Convert simple text file to learnables. Put each learnable on a line, seperate the front and
/// back with a '|' character.
#[derive(Parser, Debug)]
#[clap(long_about = None)]
struct Args {
    /// The starting number (inclusive).
    #[clap(short, long)]
    output: String,

    /// Name to associate with the output
    #[clap(short, long)]
    name: Option<String>,

    /// The transform described in human terms.
    #[clap(short, long)]
    transform: Option<String>,

    /// The files to read.
    #[clap(required = true)]
    inputs: Vec<String>,
}

fn str_to_hash(v: &str) -> u128 {
    use md5::{Digest, Md5};

    let mut hasher = Md5::new();
    hasher.update(v);
    let result: [u8; 16] = hasher.finalize().into();

    u128::from_le_bytes(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut learnables = vec![];

    let transform_text = args
        .transform
        .unwrap_or(String::from("Hopefully you know what to do..."));
    let transform = TextTransform::new(&transform_text, TransformId(str_to_hash(&transform_text)));

    for input in args.inputs.iter() {
        use std::io::BufRead;
        let file = std::fs::File::open(input)?;
        let lines = std::io::BufReader::new(file)
            .lines()
            .map(|v| v.expect("non unicode?"))
            .collect::<Vec<String>>();
        for line in lines.iter() {
            let entries = line
                .split("|")
                .map(|v| v.to_owned())
                .collect::<Vec<String>>();
            if entries.len() < 2 {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to find two entries in : {line}"),
                )));
            }
            let mut edges = vec![];
            let t1 =
                TextRepresentation::new(&entries[0], RepresentationId(str_to_hash(&entries[0])));
            let t2 =
                TextRepresentation::new(&entries[1], RepresentationId(str_to_hash(&entries[1])));
            edges.push((t1, transform.clone(), t2));
            let learnable_id = str_to_hash(&(transform_text.clone() + line));

            learnables.push(TextLearnable::new(&(edges[..]), LearnableId(learnable_id)));
        }
    }

    save_text_learnables(&args.output, &format!("some example name."), &learnables)?;
    Ok(())
}
