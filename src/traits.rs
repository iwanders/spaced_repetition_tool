use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum RepresentationType {
    Text,
}

pub type Id = u128;
pub type Score = f64;

pub type MemorizerError = Box<dyn std::error::Error>;

/// A partciular representation.
pub trait Representation: std::fmt::Debug {
    /// Get the type of this presentation.
    fn get_type(&self) -> RepresentationType;

    /// Get the textual representation.
    fn get_text(&self) -> &str;

    /// Unique id for this representation.
    fn get_id(&self) -> Id;

    /// Check if this representation is identical to another representation.
    /// This should not compare ID, instead it should compare the contents of the representation.
    fn is_equal(&self, other: &dyn Representation) -> bool;

    /// Get the approximate equality of this representation and the other representation. Must be
    /// between 0.0 (completely wrong) and 1.0 (exactly equal).
    /// This should not compare ID, instead it should compare the contents of the representation.
    fn get_similarity(&self, other: &dyn Representation) -> Score {
        if self.is_equal(other) {
            1.0
        } else {
            0.0
        }
    }
}

/// A transformation, like Hex->Binary or 'Translate from A into B'
pub trait Transformation: std::fmt::Debug {
    /// A string describing the particular transformation to be performed.
    fn get_description(&self) -> &str;

    /// Unique id for this Transformation.
    fn get_id(&self) -> Id;
}

pub type LearnableEdge<'a> = (
    &'a dyn Representation,
    &'a dyn Transformation,
    &'a dyn Representation,
);

/// Something that relates transformations and representations to each other. This owns the
/// representations and transforms.
pub trait Learnable: std::fmt::Debug {
    /// Get the possible edges for this learnable.
    fn get_edges(&self) -> Vec<LearnableEdge>;

    /// Retrieve a questions' true representation.
    fn get_question(&self, question: &Question) -> LearnableEdge;

    /// Unique id for this learnable.
    fn get_id(&self) -> Id;
}

#[derive(Debug, PartialEq, Copy, Clone, Deserialize, Serialize)]
pub struct Question {
    pub learnable: Id,

    pub from: Id,
    pub transform: Id,
    pub to: Id,
}

#[derive(Debug, PartialEq, Copy, Clone, Deserialize, Serialize)]
pub struct Record {
    pub question: Question,
    pub score: Score,
    pub time: std::time::SystemTime,
}

/// Something to track past performance.
pub trait Recorder: std::fmt::Debug {
    /// Store an answer.
    fn store_record(&mut self, record: &Record) -> Result<(), MemorizerError>;

    /// Retrieve records by a learnable id.
    fn get_records_by_learnable(&self, learnable: Id) -> Result<Vec<Record>, MemorizerError>;
}

/// The entity that decided what questions to ask. Only works on Ids.
pub trait Selector: std::fmt::Debug {
    /// Constructor, takes recorder of past event and a set of learnables.
    fn new(
        questions: &[Question],
        recorder: &dyn Recorder,
    ) -> Result<Box<dyn Selector>, MemorizerError>
    where
        Self: Sized;

    /// Retrieve a question to ask.
    fn get_question(&mut self) -> Question;

    /// Store answer to a question, not guaranteed to be in sync with get_question.
    fn store_record(&mut self, record: &Record);
}
