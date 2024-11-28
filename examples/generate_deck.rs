use memorizer::text::{save_text_learnables, TextLearnable, TextRepresentation, TextTransform};
use memorizer::traits::{Id, LearnableId, RepresentationId, Transform, TransformId};
use serde::{Deserialize, Serialize};

use clap::Parser;

/// Convert simple text file to learnables. Put each learnable on a line, seperate the front and
/// back with a '|' character.
#[derive(Parser, Debug)]
#[clap(long_about = None)]
struct Args {
    /// The output file (inclusive).
    #[clap(short, long)]
    output: String,

    /// Name to associate with the output
    #[clap(short, long)]
    name: Option<String>,

    /// The transform described in human terms, for to direction.
    #[clap(short, long)]
    transform_to: Option<String>,

    /// The transform described in human terms, for reverse direction.
    #[clap(long)]
    transform_reverse: Option<String>,

    #[clap(long)]
    include_reverse: bool,

    /// The files to read.
    #[clap(required = true)]
    inputs: Vec<String>,
}

fn str_to_hash(v: &str) -> Id {
    use md5::{Digest, Md5};

    let mut hasher = Md5::new();
    hasher.update(v);
    let result: [u8; 16] = hasher.finalize().into();

    u128::from_le_bytes(result) as Id // truncate it.
}

fn read_learnables_from_txt(
    input: &str,
    include_reverse: bool,
    transform_to: &TextTransform,
    transform_reverse: &TextTransform,
) -> Result<Vec<TextLearnable>, memorizer::traits::MemorizerError> {
    use std::io::BufRead;
    let mut learnables = vec![];
    let file = std::fs::File::open(input).map_err(|e| format!("failed to open {input}: {e}"))?;
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
        let t1 = TextRepresentation::new(&entries[0], RepresentationId(str_to_hash(&entries[0])));
        let t2 = TextRepresentation::new(&entries[1], RepresentationId(str_to_hash(&entries[1])));
        edges.push((t1.clone(), transform_to.clone(), t2.clone()));
        if include_reverse {
            edges.push((t2, transform_reverse.clone(), t1));
        }

        let learnable_id = str_to_hash(&(transform_to.description().to_owned() + line));

        learnables.push(TextLearnable::new(&(edges[..]), LearnableId(learnable_id)));
    }
    Ok(learnables)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LearnableYaml {
    from: String,
    to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeckYaml {
    include_reverse: Option<bool>,
    transform_to: Option<String>,
    transform_reverse: Option<String>,
    learnables: Vec<LearnableYaml>,
}

fn read_learnables_from_yaml(
    input: &str,
    include_reverse: bool,
    transform_to: &TextTransform,
    transform_reverse: &TextTransform,
) -> Result<Vec<TextLearnable>, memorizer::traits::MemorizerError> {
    let mut learnables = vec![];

    let file =
        std::fs::File::open(input).map_err(|e| format!("failed to open {input:?}: {e:?}"))?;
    let deck: DeckYaml = serde_yaml::from_reader(file)?;
    let transform_to = deck
        .transform_to
        .unwrap_or(transform_to.description().to_owned());

    let transform_to = TextTransform::new(&transform_to, TransformId(str_to_hash(&transform_to)));

    let transform_reverse = deck
        .transform_reverse
        .unwrap_or(transform_reverse.description().to_owned());
    let transform_reverse = TextTransform::new(
        &transform_reverse,
        TransformId(str_to_hash(&transform_reverse)),
    );

    for entry in deck.learnables {
        let mut edges = vec![];
        let t1 = TextRepresentation::new(&entry.from, RepresentationId(str_to_hash(&entry.from)));
        let t2 = TextRepresentation::new(&entry.to, RepresentationId(str_to_hash(&entry.to)));
        edges.push((t1.clone(), transform_to.clone(), t2.clone()));
        if include_reverse {
            edges.push((t2, transform_reverse.clone(), t1));
        }

        // This id is not actually used, but it should be a unique identifier for this edge.
        let learnable_id =
            str_to_hash(&(transform_to.description().to_owned() + &entry.from + &entry.to));

        learnables.push(TextLearnable::new(&(edges[..]), LearnableId(learnable_id)));
    }
    Ok(learnables)
}

fn main() -> Result<(), memorizer::traits::MemorizerError> {
    let args = Args::parse();

    let mut learnables = vec![];

    let transform_to_text = args
        .transform_to
        .unwrap_or(String::from("Hopefully you know what to do..."));
    let transform_to = TextTransform::new(
        &transform_to_text,
        TransformId(str_to_hash(&transform_to_text)),
    );

    let transform_reverse_text = args
        .transform_reverse
        .unwrap_or(String::from("Hopefully you know what to do..."));
    let transform_reverse = TextTransform::new(
        &transform_reverse_text,
        TransformId(str_to_hash(&transform_reverse_text)),
    );
    let include_reverse = args.include_reverse;

    for input in args.inputs.iter() {
        if input.ends_with("txt") {
            learnables.extend(read_learnables_from_txt(
                &input,
                include_reverse,
                &transform_to,
                &transform_reverse,
            )?);
        }

        if input.ends_with("yaml") || input.ends_with("yml") {
            learnables.extend(read_learnables_from_yaml(
                &input,
                include_reverse,
                &transform_to,
                &transform_reverse,
            )?);
        }
    }

    save_text_learnables(&args.output, &format!("some example name."), &learnables)?;
    Ok(())
}
