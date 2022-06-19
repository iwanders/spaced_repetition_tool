use crate::traits::*;
use serde::{Deserialize, Serialize};

// use serde::de::Deserializer;
// use serde::ser::Serializer;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextRepresentation {
    text: String,
    id: Id,
}

impl Representation for TextRepresentation {
    fn get_type(&self) -> RepresentationType {
        RepresentationType::Text
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextTransformation {
    text: String,
    id: Id,
}

impl Transformation for TextTransformation {
    fn get_description(&self) -> &str {
        &self.text
    }

    fn get_id(&self) -> Id {
        self.id
    }
}

type TextEdge = (TextRepresentation, TextTransformation, TextRepresentation);
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct TextLearnable {
    edges: Vec<TextEdge>,
    id: Id,
}

impl Learnable for TextLearnable {
    fn get_edges(&self) -> Vec<LearnableEdge> {
        self.edges
            .iter()
            .map(|z| {
                (
                    Box::new(&z.0 as &dyn Representation),
                    Box::new(&z.1 as &dyn Transformation),
                    Box::new(&z.2 as &dyn Representation),
                )
            })
            .collect::<_>()
    }

    fn get_id(&self) -> Id {
        self.id
    }
}

/// Representation on disk. Very much intended to be machine readable only.
#[derive(Debug, Deserialize, Serialize)]
pub struct TextLearnableStorage {
    name: String,
    id: Id,
    transformations: Vec<TextTransformation>,
    representations: Vec<TextRepresentation>,
    learnables: Vec<Vec<(Id, Id, Id)>>,
}

pub fn load_text_learnables(
    filename: &str,
) -> Result<Vec<Box<dyn Learnable>>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(filename).expect("file should be opened");
    if filename.ends_with("yaml") {
        let yaml: serde_yaml::Value = serde_yaml::from_reader(file)?;
        let storage: TextLearnableStorage = serde_yaml::from_value(yaml)?;
        // We need to go from this storage thing into the vector of Learnables.

        // First, create two hashmaps to look up ids from.
        use std::collections::HashMap;
        let transforms = storage
            .transformations
            .iter()
            .map(|z| (z.get_id(), z.clone()))
            .collect::<HashMap<Id, TextTransformation>>();
        let representations = storage
            .representations
            .iter()
            .map(|z| (z.get_id(), z.clone()))
            .collect::<HashMap<Id, TextRepresentation>>();

        // Now, we can iterate through the learnables and connect all entries.
        let mut res: Vec<Box<dyn Learnable>> = vec![];
        for (i, relations) in storage.learnables.iter().enumerate() {
            let mut learnable: TextLearnable = TextLearnable { id:i as Id + storage.id, ..Default::default()};
            for (r1, t, r2) in relations.iter() {
                let repr1 = representations.get(r1).ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to find representation: {r1}"),
                    ))
                })?;
                let repr2 = representations.get(r2).ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to find representation: {r2}"),
                    ))
                })?;
                let tr = transforms.get(t).ok_or_else(|| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to find transform: {t}"),
                    ))
                })?;
                learnable.edges.push((repr1.clone(), tr.clone(), repr2.clone()));
            }
            res.push(Box::new(learnable));
        }

        return Ok(res);
    }
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "File type not supported. Use .yaml.",
    )))
}
