pub enum RepresentationType {
    Text,
}

/// A partciular representation.
pub trait Representation: std::fmt::Debug {
    /// Get the type of this presentation.
    fn get_type(&self) -> RepresentationType;

    /// Get the textual representation.
    fn get_text(&self) -> &str;

    /// Unique id for this representation.
    fn get_id(&self) -> u128;
}

/// A transformation, like Hex->Binary or 'Translate from A into B'
pub trait Transformation: std::fmt::Debug {
    /// A string describing the particular transformation to be performed.
    fn get_description(&self) -> &str;

    /// Unique id for this Transformation.
    fn get_id(&self) -> u128;
}

/// Something that relates transformations and representations to each other.
pub trait Learnable: std::fmt::Debug {
    /// Get the possible edges for this learnable.
    fn get_edges(
        &self,
    ) -> &[(
        &dyn Representation,
        &dyn Transformation,
        &dyn Representation,
    )];

    /// Unique id for this learnable.
    fn get_id(&self) -> u128;
}
