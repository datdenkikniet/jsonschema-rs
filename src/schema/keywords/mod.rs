mod logic;
pub use logic::{LogicApplier, LogicValidationError};

mod property;
pub use property::Property;

mod ty;
pub use ty::Type;

pub mod annotations {
    pub use super::logic::{LogicError, LogicErrorKind};
    pub use super::property::{PropertyError, PropertyErrorKind};
    pub use super::ty::{TypeError, TypeErrorKind};
}
