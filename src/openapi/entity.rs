use crate::{Form, Json, Query};
pub use rweb_openapi::v3_0::*;
use std::collections::BTreeMap;

/// This can be derived by `#[derive(Schema)]`.
///
///
/// You may provide an example value of each field with
///
/// `#[schema(example = "path_to_function")]`
pub trait Entity {
    fn describe() -> Schema;

    fn describe_response() -> Response {
        let schema = Self::describe();
        let mut content = BTreeMap::new();
        content.insert(
            // TODO
            "*/*".into(),
            MediaType {
                schema: Some(ObjectOrReference::Object(schema)),
                examples: None,
                encoding: Default::default(),
            },
        );
        Response {
            content,
            ..Default::default()
        }
    }
}

impl<T: Entity> Entity for Vec<T> {
    fn describe() -> Schema {
        Schema {
            schema_type: "array".into(),
            items: Some(Box::new(T::describe())),
            ..Default::default()
        }
    }
}

impl<T> Entity for Option<T>
where
    T: Entity,
{
    fn describe() -> Schema {
        let mut s = T::describe();
        s.nullable = Some(true);
        s
    }
}

impl<T> Entity for Json<T>
where
    T: Entity,
{
    #[inline]
    fn describe() -> Schema {
        T::describe()
    }

    fn describe_response() -> Response {
        let schema = Self::describe();
        let mut content = BTreeMap::new();
        content.insert(
            "application/json".into(),
            MediaType {
                schema: Some(ObjectOrReference::Object(schema)),
                examples: None,
                encoding: Default::default(),
            },
        );
        Response {
            content,
            ..Default::default()
        }
    }
}

impl<T> Entity for Query<T>
where
    T: Entity,
{
    #[inline]
    fn describe() -> Schema {
        T::describe()
    }
}

impl<T> Entity for Form<T>
where
    T: Entity,
{
    #[inline]
    fn describe() -> Schema {
        T::describe()
    }
}

macro_rules! number {
    ($T:ty) => {
        impl Entity for $T {
            #[inline]
            fn describe() -> Schema {
                Schema {
                    schema_type: "number".to_string(),
                    ..Default::default()
                }
            }
        }

    };

    (
        $(
            $T:ty
        ),*
    ) => {
        $(
            number!($T);
        )*
    };
}

number!(u8, u16, u32, u64, u128, usize);
number!(i8, i16, i32, i64, i128, isize);
// TODO: non-zero types

impl Entity for String {
    fn describe() -> Schema {
        Schema {
            schema_type: "string".to_string(),
            ..Default::default()
        }
    }
}
