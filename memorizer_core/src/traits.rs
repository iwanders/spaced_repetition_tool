

pub enum RepresentationType
{
    Text,
}

/// A partciular representation.
pub trait Representation : std::fmt::Debug{
    fn get_type(&self) -> RepresentationType;
    fn get_text(&self) -> &str;
}

/// A transformation, like Hex->Binary or 'Translate from A into B'
pub trait Transformation : std::fmt::Debug {
    /// A string describing the particular transformation to be performed.
    fn get_description(&self) -> &str;
}

/// Something that relates transformations and representations to each other.
pub trait Learnable : std::fmt::Debug {
    fn get_edges(&self) -> &[(&dyn Representation, &dyn Transformation, &dyn Representation)];
}

