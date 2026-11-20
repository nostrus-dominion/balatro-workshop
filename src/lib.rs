pub mod codec;
pub mod deck;
pub mod discovery;
pub mod document;
pub mod fields;
pub mod jokers;
pub mod lua;

pub type Result<T> = std::result::Result<T, String>;
