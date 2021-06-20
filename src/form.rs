use crate::openapi::{Components, Entity};
use rweb_openapi::v3_0::Schema;
use serde::{de::DeserializeOwned, Deserialize};
use validator::{Validate, ValidationError};

pub struct Valid<T>
where
    T: DeserializeOwned + Validate,
{
    inner: Result<T, ValidationError>,
}

impl<T> Valid<T>
where
    T: DeserializeOwned + Validate,
{
    pub fn value(self) -> Result<T, ValidationError> {
        self.inner
    }
}

impl<T> Entity for Valid<T>
where
    T: DeserializeOwned + Validate + Entity,
{
    fn describe() -> Schema {
        T::describe()
    }

    fn describe_components() -> Components {
        T::describe_components()
    }
}

impl<'de, T> Deserialize<'de> for Valid<T>
where
    T: DeserializeOwned + Validate,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}
