use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum RepresentationType {
    Text,
}

pub type Id = u64;

/// Id for a learnable, a learnable represents a set of learnable edges.
/// Think of a normal flashcard as a single learnable with two edges (back to front, front to back)
#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Copy, Clone, Default)]
#[serde(transparent)]
pub struct LearnableId(pub Id);

/// Id for a representation, this is a unique id for the front or back of a traditional card.
#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Copy, Clone, Default)]
#[serde(transparent)]
pub struct RepresentationId(pub Id);

/// Id for a particular transformation, like translating language A to B.
#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize, Copy, Clone, Default)]
#[serde(transparent)]
pub struct TransformId(pub Id);

/// Type for the score, in interval [0.0 to 1.0] (inclusive).
pub type Score = f64;

/// Error in case anything goes wrong.
pub type MemorizerError = Box<dyn std::error::Error>;

/// A particular representation of data, think about the side of a card.
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

/// A transformation, like Hex->Binary or 'Translate from A into B', think about the direction
/// the card is being learnt in.
pub trait Transform: std::fmt::Debug {
    /// A string describing the particular transformation to be performed.
    fn description(&self) -> &str;

    /// Unique id for this Transformation.
    fn id(&self) -> TransformId;
}

/// A struct representing a particular question.
#[derive(Debug, PartialEq, Copy, Clone, Deserialize, Serialize, Default)]
pub struct Question {
    /// From which learnable this question originates.
    pub learnable: LearnableId,

    /// The 'from' value, this is shown to the user.
    pub from: RepresentationId,

    /// The transformation the user is to perform.
    pub transform: TransformId,

    /// The true answer for this from and transformation.
    pub to: RepresentationId,
}

/// Something that relates transformations and representations to each other. This owns the
/// representations and transforms. Think of this as a single card with front and back (or any
/// number of sides).
pub trait Learnable: std::fmt::Debug {
    /// Get the possible edges for this learnable.
    fn edges(&self) -> Vec<Question>;

    /// Retrieval function for a particular representation. Panics if the id is not known to this
    /// learnable.
    fn representation(&self, id: RepresentationId) -> Rc<dyn Representation>;

    /// Retrieval function for a particular transformation. Panics if the id is not known to this
    /// learnable.
    fn transform(&self, transform: TransformId) -> Rc<dyn Transform>;

    /// Unique id for this learnable.
    fn id(&self) -> LearnableId;
}

/// Record of an question, the score obtained answering it and a timestamp.
#[derive(Debug, PartialEq, Copy, Clone, Deserialize, Serialize)]
pub struct Record {
    /// The question as posed.
    pub question: Question,

    /// The final score stored for this question.
    pub score: Score,

    /// Timestamp associated to this question.
    pub time: std::time::SystemTime,
}

/// Something to track past performance.
pub trait Recorder: std::fmt::Debug {
    /// Store an answer.
    fn store_record(&mut self, record: &Record) -> Result<(), MemorizerError>;

    /// Retrieve records for a particular question.
    fn get_records_by_question(
        &self,
        question: &Question,
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
