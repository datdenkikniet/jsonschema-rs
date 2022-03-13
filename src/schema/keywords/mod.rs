mod logic;
pub use logic::{LogicApplier, LogicValidationError};

mod object;
pub use object::Property;

mod ty;
pub use ty::{PrimitiveType, Type};

mod array;
pub use array::{Items, PrefixItems};

mod enum_kw;
pub use enum_kw::Enum;

macro_rules! get_if_is {
    ($input: expr, $annotations: expr, $is: path, $err: expr) => {
        match $input {
            $is(val) => val,
            _ => {
                $annotations.push($err);
                return false;
            },
        }
    };
}

pub(crate) use get_if_is;


pub mod annotations {
    pub use super::enum_kw::EnumError;
    pub use super::array::{ItemsError, ItemsErrorKind};
    pub use super::logic::{LogicError, LogicErrorKind};
    pub use super::object::{PropertyError, PropertyErrorKind};
    pub use super::ty::TypeError;
}
