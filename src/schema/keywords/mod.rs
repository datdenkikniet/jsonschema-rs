mod logic;
pub use logic::{LogicApplier, LogicValidationError};

mod property;
pub use property::Property;

mod ty;
pub use ty::Type;

mod enum_kw;
pub use enum_kw::Enum;

pub mod annotations {
    pub use super::enum_kw::EnumError;
    pub use super::logic::{LogicError, LogicErrorKind};
    pub use super::property::{PropertyError, PropertyErrorKind};
    pub use super::ty::{TypeError, TypeErrorKind};
}
