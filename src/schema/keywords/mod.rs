mod logic;
pub use logic::{LogicApplier, LogicValidationError};

mod object;
pub use object::Property;

mod ty;
pub use ty::{PrimitiveType, Type};

mod array;
pub use array::{Contains, Items, PrefixItems};

mod enum_kw;
pub use enum_kw::Enum;

pub mod annotations {
    pub use super::array::{ArrayError, ArrayErrorKind};
    pub use super::enum_kw::EnumError;
    pub use super::logic::{LogicError, LogicErrorKind};
    pub use super::object::{PropertyError, PropertyErrorKind};
    pub use super::ty::TypeError;
}
