use crate::traits::*;

#[derive(Debug)]
struct TextRepresentation {
    text: String,
    id: u128,
}

impl Representation for TextRepresentation {
    fn get_type(&self) -> RepresentationType {
        RepresentationType::Text
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn get_id(&self) -> u128 {
        self.id
    }
}

#[derive(Debug)]
struct TextTransformation {
    text: String,
    id: u128,
}

impl Transformation for TextTransformation {
    fn get_description(&self) -> &str {
        &self.text
    }

    fn get_id(&self) -> u128 {
        self.id
    }
}

type TextEdge = (TextRepresentation, TextTransformation, TextRepresentation);
#[derive(Debug)]
struct TextLearnable {
    edges: Vec<TextEdge>,
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

    fn get_id(&self) -> u128 {
        0
    }
}
