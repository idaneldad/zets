//! Canonization errors.

use crate::sense_graph::LanguageId;
use std::fmt;

#[derive(Debug)]
pub enum CanonizationError {
    InvalidInput(String),
    LanguageNotSupported(LanguageId),
    FingerprintFailed,
    ClassificationFailed,
}

impl fmt::Display for CanonizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Self::LanguageNotSupported(id) => write!(f, "language not supported: {id}"),
            Self::FingerprintFailed => write!(f, "fingerprint computation failed"),
            Self::ClassificationFailed => write!(f, "classification failed"),
        }
    }
}
