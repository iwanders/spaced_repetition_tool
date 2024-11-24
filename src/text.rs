use crate::traits::*;
use serde::{Deserialize, Serialize};

/// Simplest implementation for a text representation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextRepresentation {
    text: String,
    id: RepresentationId,
}

impl TextRepresentation {
    pub fn new(text: &str, id: RepresentationId) -> Self {
        TextRepresentation {
            text: text.to_owned(),
            id,
        }
    }

    pub fn from(other: std::sync::Arc<dyn Representation>) -> Self {
        TextRepresentation {
            text: other.text().to_string(),
            id: other.id(),
        }
    }
}

impl Representation for TextRepresentation {
    fn get_type(&self) -> RepresentationType {
        RepresentationType::Text
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn id(&self) -> RepresentationId {
        self.id
    }

    fn is_equal(&self, other: &dyn Representation) -> bool {
        self.get_type() == other.get_type() && self.text() == other.text()
    }
}

/// Simplest implementation for a text transformation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextTransform {
    text: String,
    id: TransformId,
}

impl TextTransform {
    pub fn new(text: &str, id: TransformId) -> Self {
        TextTransform {
            text: text.to_owned(),
            id,
        }
    }
    pub fn from(other: std::sync::Arc<dyn Transform>) -> Self {
        TextTransform {
            text: other.description().to_string(),
            id: other.id(),
        }
    }
}

impl Transform for TextTransform {
    fn description(&self) -> &str {
        &self.text
    }

    fn id(&self) -> TransformId {
        self.id
    }
}

type TextEdge = (TextRepresentation, TextTransform, TextRepresentation);

/// A text learnable holds several learnables.
#[derive(Debug, Default, Clone)]
pub struct TextLearnable {
    representations:
        std::collections::HashMap<RepresentationId, std::sync::Arc<TextRepresentation>>,
    transforms: std::collections::HashMap<TransformId, std::sync::Arc<TextTransform>>,
    edges: Vec<Question>,
    id: LearnableId,
}
impl TextLearnable {
    pub fn new(edges: &[TextEdge], id: LearnableId) -> Self {
        let mut res = TextLearnable {
            id,
            ..Default::default()
        };
        for (r1, transform, r2) in edges.iter() {
            res.representations
                .insert(r1.id(), std::sync::Arc::new(r1.clone()));
            res.representations
                .insert(r2.id(), std::sync::Arc::new(r2.clone()));
            res.transforms
                .insert(transform.id(), std::sync::Arc::new(transform.clone()));
            res.edges.push(Question {
                learnable: id,
                from: r1.id(),
                transform: transform.id(),
                to: r2.id(),
            });
        }
        res
    }
}
impl Learnable for TextLearnable {
    fn edges(&self) -> Vec<Question> {
        self.edges.clone()
    }

    fn representation(&self, id: RepresentationId) -> std::sync::Arc<dyn Representation> {
        self.representations
            .get(&id)
            .expect("Requested id must exist")
            .clone()
    }

    fn transform(&self, id: TransformId) -> std::sync::Arc<dyn Transform> {
        self.transforms
            .get(&id)
            .expect("Requested id must exist")
            .clone()
    }

    /// Unique id for this learnable.
    fn id(&self) -> LearnableId {
        self.id
    }
}

/// Representation on disk. Very much intended to be machine readable only.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TextLearnableStorage {
    name: String,
    transformations: Vec<TextTransform>,
    representations: Vec<TextRepresentation>,
    learnables: Vec<Vec<(RepresentationId, TransformId, RepresentationId)>>,
}

pub fn load_text_learnables(
    filename: &str,
) -> Result<Vec<Box<dyn Learnable>>, Box<dyn std::error::Error + Send + Sync>> {
    let file =
        std::fs::File::open(filename).map_err(|e| format!("failed to open {filename}: {e}"))?;
    if filename.ends_with("yaml") {
        let yaml: serde_yaml::Value = serde_yaml::from_reader(file)?;
        let storage: TextLearnableStorage = serde_yaml::from_value(yaml)?;
        // We need to go from this storage thing into the vector of Learnables.

        // First, create two hashmaps to look up ids from.
        use std::collections::HashMap;
        let transforms = storage
            .transformations
            .iter()
            .map(|z| (z.id(), z.clone()))
            .collect::<HashMap<TransformId, TextTransform>>();
        let representations = storage
            .representations
            .iter()
            .map(|z| (z.id(), z.clone()))
            .collect::<HashMap<RepresentationId, TextRepresentation>>();

        // Now, we can iterate through the learnables and connect all entries.
        let mut res: Vec<Box<dyn Learnable>> = vec![];
        for (i, relations) in storage.learnables.iter().enumerate() {
            let mut edges = vec![];

            for (r1, t, r2) in relations.iter() {
                let repr1 = representations.get(r1).ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to find representation: {r1:?}"),
                    ))
                })?;
                let repr2 = representations.get(r2).ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to find representation: {r2:?}"),
                    ))
                })?;
                let tr = transforms.get(t).ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to find transform: {t:?}"),
                    ))
                })?;
                edges.push((repr1.clone(), tr.clone(), repr2.clone()));
            }
            let learnable = TextLearnable::new(&edges, LearnableId(i as Id));

            res.push(Box::new(learnable));
        }

        return Ok(res);
    }
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "File type not supported. Use .yaml.",
    )))
}

pub fn save_text_learnables(
    filename: &str,
    name: &str,
    learnables: &[TextLearnable],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut storage = TextLearnableStorage {
        name: name.to_owned(),
        ..Default::default()
    };
    use std::collections::BTreeMap;
    let mut transforms: BTreeMap<TransformId, TextTransform> = Default::default();
    let mut representations: BTreeMap<RepresentationId, TextRepresentation> = Default::default();
    for learnable in learnables.iter() {
        let mut edges = vec![];
        for q in learnable.edges.iter() {
            representations.insert(
                q.from,
                TextRepresentation::from(learnable.representation(q.from)),
            );
            representations.insert(
                q.to,
                TextRepresentation::from(learnable.representation(q.to)),
            );
            transforms.insert(
                q.transform,
                TextTransform::from(learnable.transform(q.transform)),
            );
            edges.push((q.from, q.transform, q.to));
        }
        storage.learnables.push(edges);
    }
    for (_id, tr) in transforms {
        storage.transformations.push(tr);
    }

    for (_id, repr) in representations {
        storage.representations.push(repr);
    }
    // let yaml: serde_yaml::Value = serde_yaml::from_reader(file)?;
    // let storage: TextLearnableStorage = serde_yaml::from_value(yaml)?;
    use std::fs::OpenOptions;
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)?;
    serde_yaml::to_writer(file, &storage)?;

    Ok(())
}
