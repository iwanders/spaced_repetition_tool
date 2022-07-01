use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum RepresentationType {
    Text,
}

pub type Id = u128;

#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Copy, Clone, Default)]
#[serde(transparent)]
pub struct LearnableId(pub Id);

#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Copy, Clone, Default)]
#[serde(transparent)]
pub struct RepresentationId(pub Id);

#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Copy, Clone, Default)]
#[serde(transparent)]
pub struct TransformId(pub Id);

pub type Score = f64;

pub type MemorizerError = Box<dyn std::error::Error>;

/// A partciular representation.
pub trait Representation: std::fmt::Debug {
    /// Get the type of this presentation.
    fn get_type(&self) -> RepresentationType;

    /// Get the textual representation.
    fn text(&self) -> &str;

    /// Unique id for this representation.
    fn id(&self) -> RepresentationId;

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
pub trait Transform: std::fmt::Debug {
    /// A string describing the particular transformation to be performed.
    fn description(&self) -> &str;

    /// Unique id for this Transformation.
    fn id(&self) -> TransformId;
}

#[derive(Debug, PartialEq, Copy, Clone, Deserialize, Serialize, Default)]
pub struct Question {
    pub learnable: LearnableId,

    pub from: RepresentationId,
    pub transform: TransformId,
    pub to: RepresentationId,
}

/// Something that relates transformations and representations to each other. This owns the
/// representations and transforms.
pub trait Learnable: std::fmt::Debug {
    /// Get the possible edges for this learnable.
    fn edges(&self) -> Vec<Question>;

    fn representation(&self, id: RepresentationId) -> Rc<dyn Representation>;
    fn transform(&self, transform: TransformId) -> Rc<dyn Transform>;

    /// Unique id for this learnable.
    fn id(&self) -> LearnableId;
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
    fn get_records_by_learnable(
        &self,
        learnable: LearnableId,
    ) -> Result<Vec<Record>, MemorizerError>;
}

/// The entity that decided what questions to ask. Only works on Ids.
pub trait Selector: std::fmt::Debug {
    /// Constructor, takes recorder of past event and a set of learnables.
    fn set_questions(&mut self, questions: &[Question], recorder: &dyn Recorder);

    /// Retrieve a question to ask, if empty session is done, no questions to ask right now.
    fn get_question(&mut self) -> Option<Question>;

    /// Store answer to a question, not guaranteed to be in sync with get_question.
    fn store_record(&mut self, record: &Record);
}
