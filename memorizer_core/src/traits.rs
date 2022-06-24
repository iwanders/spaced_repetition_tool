#[derive(Debug, PartialEq)]
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
    Box<&'a dyn Representation>,
    Box<&'a dyn Transformation>,
    Box<&'a dyn Representation>,
);

/// Something that relates transformations and representations to each other.
pub trait Learnable: std::fmt::Debug {
    /// Get the possible edges for this learnable.
    fn get_edges(&self) -> Vec<LearnableEdge>;

    /// Unique id for this learnable.
    fn get_id(&self) -> Id;
}



#[derive(Debug, PartialEq)]
pub struct Record {
    pub learnable: Id,

    pub from: Id,
    pub transform: Id,
    pub to: Id,

    pub score: Score,
    pub time: std::time::SystemTime,
}

pub trait RecordKeeper : std::fmt::Debug {
    /// Store an answer.
    fn store_record(&mut self, record: &Record) -> Result<(), MemorizerError>;

    /// Retrieve records by a learnable id.
    fn get_records_by_id(&self, learnable: Id) -> Result<Vec<Record>, MemorizerError>;
}
