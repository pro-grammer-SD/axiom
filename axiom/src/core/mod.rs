pub mod value;
pub mod oop;

pub use value::{AxValue, AxObject, ValidationError};
pub use oop::{AxCallable, AxClass, AxInstance, AxEnum, AxEnumVariantDef};
