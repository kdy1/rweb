use crate::{Form, Json, Query};
pub use rweb_openapi::v3_0::*;
use std::{borrow::Cow, collections::BTreeMap};

/// This can be derived by `#[derive(Schema)]`.
///
///
/// You may provide an example value of each field with
///
/// `#[schema(example = "path_to_function")]`
pub trait Entity {
    fn describe() -> Schema;

    fn describe_component() -> Option<(Cow<'static, str>, Schema)> {
        None
    }

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
            schema_type: Type::Array,
            items: Some(Box::new(T::describe())),
            ..Default::default()
        }
    }

    fn describe_component() -> Option<(Cow<'static, str>, Schema)> {
        let (name, s) = T::describe_component()?;
        Some((
            Cow::Owned(format!("{}List", name)),
            Schema {
                schema_type: Type::Array,
                items: Some(Box::new(s)),
                ..Default::default()
            },
        ))
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

    fn describe_component() -> Option<(Cow<'static, str>, Schema)> {
        let (k, mut s) = T::describe_component()?;
        s.nullable = Some(true);
        Some((k, s))
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

    fn describe_component() -> Option<(Cow<'static, str>, Schema)> {
        T::describe_component()
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

    fn describe_component() -> Option<(Cow<'static, str>, Schema)> {
        T::describe_component()
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

    fn describe_component() -> Option<(Cow<'static, str>, Schema)> {
        T::describe_component()
    }
}

macro_rules! integer {
    ($T:ty) => {
        impl Entity for $T {
            #[inline]
            fn describe() -> Schema {
                Schema {
                    schema_type: Type::Integer,
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
            integer!($T);
        )*
    };
}

integer!(u8, u16, u32, u64, u128, usize);
integer!(i8, i16, i32, i64, i128, isize);
// TODO: non-zero types

impl Entity for String {
    fn describe() -> Schema {
        Schema {
            schema_type: Type::String,
            ..Default::default()
        }
    }
}
