//! Main memorizer functionality.

// Want to be able to do cards like front <-> back, the traditional.
// But also 0x23 <-> 35 <-> 0b100011, so a triangle of representations.
// Or even 0x23 <-> 35 <-> 0b100011 <-> '#', a triangle...

/// Main traits
pub mod traits;

// / Simple implementation for text
pub mod text;

// / Simple implementation to keep records
pub mod recorder;

// / Algorithm things.
pub mod algorithm;

// / Implementor for a training loop.
pub mod training;
