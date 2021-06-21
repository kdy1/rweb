use crate::{Form, Json, Query};
use indexmap::IndexMap;
pub use rweb_openapi::v3_0::*;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    convert::Infallible,
    sync::Arc,
};
use warp::{Rejection, Reply};

pub type Components = Vec<(Cow<'static, str>, Schema)>;

pub type Responses = IndexMap<Cow<'static, str>, Response>;

/// This can be derived by `#[derive(Schema)]`.
///
/// # `#[derive(Schema)]`
///
/// It implements [Entity] for the struct or enum. Note that it's recommended to
/// use `derive(Schema)` even when you are not using openapi, as it is noop when
/// cargo feature openapi is disabled.
///
/// ## Overriding description
///
/// ```rust
/// use rweb::*;
///
/// /// private documentation, for example
/// #[derive(Debug, Default, Schema)]
/// // #[schema(description = "This is output!!")]
/// pub struct Output {
///     /// By default, doc comments become description
///     data: String,
///     /// Another private info like implementation detail.
///     #[schema(description = "field")]
///     field_example: String,
/// }
/// ```
///
/// ## Component
///
/// ```rust
/// use rweb::*;
/// use serde::{Serialize, Deserialize};
///
/// // This item is stored at #/components/schema/Item
/// #[derive(Debug, Serialize, Deserialize, Schema)]
/// #[schema(component = "Item")]
/// struct ComponentTestReq {
///     data: String,
/// }
/// ```
///
/// ## Example value
///
/// `#[schema(example = $path)]` is supported. If `$path` is a literal, it's
/// automatically converted into json value. Otherwise, you should provide an
/// expression to get example value.
///
///
/// ```rust
/// use rweb::*;
/// use serde::{Serialize, Deserialize};
///
/// // This item is stored at #/components/schema/Item
/// #[derive(Debug, Serialize, Deserialize, Schema)]
/// struct ExampleTest {
///     #[schema(example = "10")]
///     data: usize,
///     #[schema(example = "\"Example for string values must be escaped like this\"")]
///     s: String,
///     #[schema(example = "complex_example()")]
///     complex: String,
/// }
///
/// fn complex_example() -> serde_json::Value {
///     serde_json::Value::String(String::from("this is example!"))
/// }
/// ```
pub trait Entity {
    fn describe() -> Schema;

    fn describe_components() -> Components {
        Default::default()
    }
}

/// This should be implemented only for types that know how it should be
/// encoded.
pub trait ResponseEntity: Entity {
    fn describe_responses() -> Responses;
}

/// Implements Entity with an empty return value.
macro_rules! empty_entity {
    ($T:ty) => {
        impl Entity for $T {
            fn describe() -> Schema {
                <() as Entity>::describe()
            }
        }
    };
}

impl Entity for () {
    /// Returns empty schema
    #[inline]
    fn describe() -> Schema {
        Schema {
            schema_type: Some(Type::Object),
            ..Default::default()
        }
    }
}

macro_rules! integer {
    ($T:ty) => {
        impl Entity for $T {
            #[inline]
            fn describe() -> Schema {
                Schema {
                    schema_type: Some(Type::Integer),
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

macro_rules! number {
    ($T:ty) => {
        impl Entity for $T {
            #[inline]
            fn describe() -> Schema {
                Schema {
                    schema_type: Some(Type::Number),
                    ..Default::default()
                }
            }
        }
    };
}

number!(f32);
number!(f64);

impl Entity for bool {
    #[inline]
    fn describe() -> Schema {
        Schema {
            schema_type: Some(Type::Boolean),
            ..Default::default()
        }
    }
}

impl Entity for char {
    #[inline]
    fn describe() -> Schema {
        Schema {
            schema_type: Some(Type::String),
            ..Default::default()
        }
    }
}

impl Entity for str {
    #[inline]
    fn describe() -> Schema {
        Schema {
            schema_type: Some(Type::String),
            ..Default::default()
        }
    }
}

impl ResponseEntity for str {
    fn describe_responses() -> Responses {
        String::describe_responses()
    }
}

impl<T> Entity for Box<T>
where
    T: ?Sized + Entity,
{
    fn describe() -> Schema {
        T::describe()
    }

    fn describe_components() -> Components {
        T::describe_components()
    }
}

impl<T> ResponseEntity for Box<T>
where
    T: ?Sized + ResponseEntity,
{
    fn describe_responses() -> Responses {
        T::describe_responses()
    }
}

impl<T> Entity for Arc<T>
where
    T: ?Sized + Entity,
{
    fn describe() -> Schema {
        T::describe()
    }

    fn describe_components() -> Components {
        T::describe_components()
    }
}

impl<T> ResponseEntity for Arc<T>
where
    T: ?Sized + ResponseEntity,
{
    fn describe_responses() -> Responses {
        T::describe_responses()
    }
}

impl<'a, T> Entity for &'a T
where
    T: ?Sized + Entity,
{
    fn describe() -> Schema {
        T::describe()
    }

    fn describe_components() -> Components {
        T::describe_components()
    }
}

impl<'a, T> ResponseEntity for &'a T
where
    T: ?Sized + ResponseEntity,
{
    fn describe_responses() -> Responses {
        T::describe_responses()
    }
}

impl<T: Entity> Entity for Vec<T> {
    fn describe() -> Schema {
        <[T] as Entity>::describe()
    }

    fn describe_components() -> Components {
        <[T] as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for HashMap<String, T> {
    fn describe() -> Schema {
        let s = <T as Entity>::describe();
        if s.ref_path.is_empty() {
            Schema {
                schema_type: Some(Type::Object),
                additional_properties: Some(ObjectOrReference::Object(Box::new(s))),
                ..Default::default()
            }
        } else {
            Schema {
                ref_path: Cow::Owned(format!("{}_Map", s.ref_path)),
                ..Default::default()
            }
        }
    }

    fn describe_components() -> Components {
        let mut v = T::describe_components();
        let s = T::describe();
        if !s.ref_path.is_empty() {
            let cn = &s.ref_path[("#/components/schemas/".len())..];
            v.push((
                Cow::Owned(format!("{}_Map", cn)),
                Schema {
                    schema_type: Some(Type::Object),
                    additional_properties: Some(ObjectOrReference::Object(Box::new(s))),
                    ..Default::default()
                },
            ));
        }
        v
    }
}

impl<T: Entity> Entity for [T] {
    fn describe() -> Schema {
        let s = T::describe();
        if s.ref_path.is_empty() {
            Schema {
                schema_type: Some(Type::Array),
                items: Some(Box::new(s)),
                ..Default::default()
            }
        } else {
            Schema {
                ref_path: Cow::Owned(format!("{}_List", s.ref_path)),
                ..Default::default()
            }
        }
    }

    fn describe_components() -> Components {
        let mut v = T::describe_components();
        let s = T::describe();
        if !s.ref_path.is_empty() {
            let cn = &s.ref_path[("#/components/schemas/".len())..];
            v.push((
                Cow::Owned(format!("{}_List", cn)),
                Schema {
                    schema_type: Some(Type::Array),
                    items: Some(Box::new(s)),
                    ..Default::default()
                },
            ));
        }
        v
    }
}

impl<T: Entity, const N: usize> Entity for [T; N] {
    fn describe() -> Schema {
        let s = T::describe();
        if s.ref_path.is_empty() {
            Schema {
                schema_type: Some(Type::Array),
                items: Some(Box::new(s)),
                min_items: Some(N),
                max_items: Some(N),
                ..Default::default()
            }
        } else {
            Schema {
                ref_path: Cow::Owned(format!("{}_Array_{}", s.ref_path, N)),
                ..Default::default()
            }
        }
    }

    fn describe_components() -> Components {
        let mut v = T::describe_components();
        let s = T::describe();
        if !s.ref_path.is_empty() {
            let cn = &s.ref_path[("#/components/schemas/".len())..];
            v.push((
                Cow::Owned(format!("{}_Array_{}", cn, N)),
                Schema {
                    schema_type: Some(Type::Array),
                    items: Some(Box::new(s)),
                    min_items: Some(N),
                    max_items: Some(N),
                    ..Default::default()
                },
            ));
        }
        v
    }
}

impl<T: Entity> Entity for BTreeSet<T> {
    fn describe() -> Schema {
        let s = T::describe();
        if s.ref_path.is_empty() {
            Schema {
                schema_type: Some(Type::Array),
                items: Some(Box::new(s)),
                unique_items: Some(true),
                ..Default::default()
            }
        } else {
            Schema {
                ref_path: Cow::Owned(format!("{}_Set", s.ref_path)),
                ..Default::default()
            }
        }
    }

    fn describe_components() -> Components {
        let mut v = T::describe_components();
        let s = T::describe();
        if !s.ref_path.is_empty() {
            let cn = &s.ref_path[("#/components/schemas/".len())..];
            v.push((
                Cow::Owned(format!("{}_Set", cn)),
                Schema {
                    schema_type: Some(Type::Array),
                    items: Some(Box::new(s)),
                    unique_items: Some(true),
                    ..Default::default()
                },
            ));
        }
        v
    }
}

impl<T> Entity for Option<T>
where
    T: Entity,
{
    fn describe() -> Schema {
        let mut s = T::describe();
        if s.ref_path.is_empty() {
            s.nullable = Some(true);
        } else {
            s.ref_path = Cow::Owned(format!("{}_Opt", s.ref_path))
        }
        s
    }

    fn describe_components() -> Components {
        let mut v = T::describe_components();
        let s = T::describe();
        if !s.ref_path.is_empty() {
            let cn = &s.ref_path[("#/components/schemas/".len())..];
            v.push((
                Cow::Owned(format!("{}_Opt", cn)),
                Schema {
                    nullable: Some(true),
                    one_of: vec![ObjectOrReference::Object(s)],
                    ..Default::default()
                },
            ));
        }
        v
    }
}

impl<T> ResponseEntity for Option<T>
where
    T: ResponseEntity,
{
    fn describe_responses() -> Responses {
        let mut responses = T::describe_responses();
        for (_, r) in responses.iter_mut() {
            for (_, v) in r.content.iter_mut() {
                if v.schema.is_some() {
                    match v.schema.as_mut().unwrap() {
                        ObjectOrReference::Object(ref mut o) => {
                            o.nullable = Some(true);
                        }
                        ObjectOrReference::Ref { .. } => {}
                    }
                }
            }
        }

        responses
    }
}

impl Entity for String {
    #[inline]
    fn describe() -> Schema {
        str::describe()
    }
}

impl ResponseEntity for String {
    fn describe_responses() -> Responses {
        let mut content = IndexMap::new();
        content.insert(
            Cow::Borrowed("text/plain"),
            MediaType {
                schema: Some(ObjectOrReference::Object(Self::describe())),
                examples: None,
                encoding: Default::default(),
            },
        );

        let mut map = IndexMap::new();
        map.insert(
            Cow::Borrowed("200"),
            Response {
                content,
                ..Default::default()
            },
        );
        map
    }
}

impl<T, E> Entity for Result<T, E>
where
    T: Entity,
    E: Entity,
{
    fn describe() -> Schema {
        Schema {
            one_of: vec![
                ObjectOrReference::Object(T::describe()),
                ObjectOrReference::Object(E::describe()),
            ],
            ..Default::default()
        }
    }

    fn describe_components() -> Components {
        let mut buf = vec![];
        buf.extend(T::describe_components());
        buf.extend(E::describe_components());
        buf
    }
}

impl<T, E> ResponseEntity for Result<T, E>
where
    T: ResponseEntity,
    E: ResponseEntity,
{
    fn describe_responses() -> IndexMap<Cow<'static, str>, Response> {
        let mut map = T::describe_responses();
        map.extend(E::describe_responses());
        map
    }
}

impl<V, S> Entity for HashSet<V, S>
where
    V: Entity,
{
    #[inline(always)]
    fn describe() -> Schema {
        <BTreeSet<V> as Entity>::describe()
    }

    #[inline(always)]
    fn describe_components() -> Components {
        <BTreeSet<V> as Entity>::describe_components()
    }
}

impl<V> Entity for LinkedList<V>
where
    V: Entity,
{
    #[inline(always)]
    fn describe() -> Schema {
        <[V] as Entity>::describe()
    }

    #[inline(always)]
    fn describe_components() -> Components {
        <[V] as Entity>::describe_components()
    }
}

impl<V> Entity for VecDeque<V>
where
    V: Entity,
{
    #[inline(always)]
    fn describe() -> Schema {
        <[V] as Entity>::describe()
    }

    #[inline(always)]
    fn describe_components() -> Components {
        <[V] as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for (T, T) {
    fn describe() -> Schema {
        <[T; 2] as Entity>::describe()
    }

    fn describe_components() -> Components {
        <[T; 2] as Entity>::describe_components()
    }
}
impl<T: Entity> Entity for (T, T, T) {
    fn describe() -> Schema {
        <[T; 3] as Entity>::describe()
    }

    fn describe_components() -> Components {
        <[T; 3] as Entity>::describe_components()
    }
}
impl<T: Entity> Entity for (T, T, T, T) {
    fn describe() -> Schema {
        <[T; 4] as Entity>::describe()
    }

    fn describe_components() -> Components {
        <[T; 4] as Entity>::describe_components()
    }
}
impl<T: Entity> Entity for (T, T, T, T, T) {
    fn describe() -> Schema {
        <[T; 5] as Entity>::describe()
    }

    fn describe_components() -> Components {
        <[T; 5] as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for HashMap<Arc<String>, T> {
    fn describe() -> Schema {
        <HashMap<String, T> as Entity>::describe()
    }

    fn describe_components() -> Components {
        <HashMap<String, T> as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for HashMap<Cow<'_, String>, T> {
    fn describe() -> Schema {
        <HashMap<String, T> as Entity>::describe()
    }

    fn describe_components() -> Components {
        <HashMap<String, T> as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for BTreeMap<String, T> {
    fn describe() -> Schema {
        <HashMap<String, T> as Entity>::describe()
    }

    fn describe_components() -> Components {
        <HashMap<String, T> as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for BTreeMap<Arc<String>, T> {
    fn describe() -> Schema {
        <BTreeMap<String, T> as Entity>::describe()
    }

    fn describe_components() -> Components {
        <BTreeMap<String, T> as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for BTreeMap<Cow<'_, String>, T> {
    fn describe() -> Schema {
        <BTreeMap<String, T> as Entity>::describe()
    }

    fn describe_components() -> Components {
        <BTreeMap<String, T> as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for IndexMap<String, T> {
    fn describe() -> Schema {
        <HashMap<String, T> as Entity>::describe()
    }

    fn describe_components() -> Components {
        <HashMap<String, T> as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for IndexMap<Arc<String>, T> {
    fn describe() -> Schema {
        <IndexMap<String, T> as Entity>::describe()
    }

    fn describe_components() -> Components {
        <IndexMap<String, T> as Entity>::describe_components()
    }
}

impl<T: Entity> Entity for IndexMap<Cow<'_, String>, T> {
    fn describe() -> Schema {
        <IndexMap<String, T> as Entity>::describe()
    }

    fn describe_components() -> Components {
        <IndexMap<String, T> as Entity>::describe_components()
    }
}

impl Entity for Infallible {
    #[inline]
    fn describe() -> Schema {
        <() as Entity>::describe()
    }

    #[inline]
    fn describe_components() -> Components {
        vec![]
    }
}

impl ResponseEntity for Infallible {
    #[inline]
    fn describe_responses() -> Responses {
        Default::default()
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

    fn describe_components() -> Components {
        T::describe_components()
    }
}

impl<T> ResponseEntity for Json<T>
where
    T: Entity,
{
    fn describe_responses() -> Responses {
        let schema = Self::describe();
        let mut content = IndexMap::new();
        content.insert(
            Cow::Borrowed("application/json"),
            MediaType {
                schema: Some(ObjectOrReference::Object(schema)),
                examples: None,
                encoding: Default::default(),
            },
        );
        let mut map = Responses::new();

        map.insert(
            Cow::Borrowed("200"),
            Response {
                content,
                ..Default::default()
            },
        );

        map
    }
}

impl Entity for serde_json::Value {
    fn describe() -> Schema {
        <() as Entity>::describe()
    }

    fn describe_components() -> Vec<(Cow<'static, str>, Schema)> {
        Default::default()
    }
}

impl ResponseEntity for serde_json::Value {
    fn describe_responses() -> Responses {
        let schema = Self::describe();
        let mut content = IndexMap::new();
        content.insert(
            Cow::Borrowed("application/json"),
            MediaType {
                schema: Some(ObjectOrReference::Object(schema)),
                examples: None,
                encoding: Default::default(),
            },
        );
        let mut map = Responses::new();

        map.insert(
            Cow::Borrowed("200"),
            Response {
                content,
                ..Default::default()
            },
        );

        map
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

    fn describe_components() -> Components {
        T::describe_components()
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

    fn describe_components() -> Components {
        T::describe_components()
    }
}

impl Entity for Rejection {
    fn describe() -> Schema {
        <() as Entity>::describe()
    }

    fn describe_components() -> Vec<(Cow<'static, str>, Schema)> {
        Default::default()
    }
}

impl ResponseEntity for Rejection {
    fn describe_responses() -> Responses {
        Default::default()
    }
}

impl Entity for http::Error {
    fn describe() -> Schema {
        <() as Entity>::describe()
    }

    fn describe_components() -> Vec<(Cow<'static, str>, Schema)> {
        Default::default()
    }
}

impl ResponseEntity for http::Error {
    fn describe_responses() -> Responses {
        Default::default()
    }
}

empty_entity!(dyn Reply);

impl ResponseEntity for dyn Reply {
    fn describe_responses() -> Responses {
        Default::default()
    }
}

#[cfg(feature = "uuid")]
impl Entity for uuid::Uuid {
    fn describe() -> Schema {
        Schema {
            schema_type: Some(Type::String),
            format: "uuid".into(),
            ..Default::default()
        }
    }
}

#[cfg(feature = "enumset")]
mod enumsetrepr {
    use super::*;
    use enumset::*;
    use serde::*;

    #[derive(Deserialize, Clone)]
    #[serde(untagged)]
    enum EnumSetRepr {
        BitFlags(u64),
        List(Vec<String>),
    }
    impl EnumSetRepr {
        fn detect<T: EnumSetType>() -> Self {
            serde_json::from_value(serde_json::to_value(EnumSet::<T>::new()).unwrap()).unwrap()
        }
    }

    // A `EnumSet<T>` can be serialized as either some number, or list of strings
    // depending on the presence of `#[enumset(serialize_as_list)]` attr.

    impl<T: EnumSetType + Entity> Entity for EnumSet<T> {
        fn describe() -> Schema {
            let s = T::describe();
            if s.ref_path.is_empty() {
                match EnumSetRepr::detect::<T>() {
                    EnumSetRepr::BitFlags(_) => Schema {
                        schema_type: Some(Type::Integer),
                        description: s.description,
                        ..Default::default()
                    },
                    EnumSetRepr::List(_) => Schema {
                        schema_type: Some(Type::Array),
                        items: Some(Box::new(s)),
                        ..Default::default()
                    },
                }
            } else {
                Schema {
                    ref_path: Cow::Owned(format!("{}_EnumSet", s.ref_path)),
                    ..Default::default()
                }
            }
        }

        fn describe_components() -> Components {
            let mut v = T::describe_components();
            let s = T::describe();
            if !s.ref_path.is_empty() {
                let cn = &s.ref_path[("#/components/schemas/".len())..];
                v.push((
                    Cow::Owned(format!("{}_EnumSet", cn)),
                    match EnumSetRepr::detect::<T>() {
                        EnumSetRepr::BitFlags(_) => Schema {
                            schema_type: Some(Type::Integer),
                            description: s.description,
                            ..Default::default()
                        },
                        EnumSetRepr::List(_) => Schema {
                            schema_type: Some(Type::Array),
                            items: Some(Box::new(s)),
                            ..Default::default()
                        },
                    },
                ));
            }
            v
        }
    }
}

#[cfg(feature = "chrono")]
mod chrono_impls {
    use chrono::TimeZone;

    use super::*;

    impl Entity for chrono::NaiveDateTime {
        fn describe() -> Schema {
            Schema {
                schema_type: Some(Type::String),
                format: "date-time".into(),
                ..Default::default()
            }
        }
    }

    impl Entity for chrono::NaiveDate {
        fn describe() -> Schema {
            Schema {
                schema_type: Some(Type::String),
                format: "date".into(),
                ..Default::default()
            }
        }
    }

    impl<T> Entity for chrono::Date<T>
    where
        T: TimeZone,
    {
        fn describe() -> Schema {
            Schema {
                schema_type: Some(Type::String),
                format: "date".into(),
                ..Default::default()
            }
        }
    }

    impl<T> Entity for chrono::DateTime<T>
    where
        T: TimeZone,
    {
        fn describe() -> Schema {
            Schema {
                schema_type: Some(Type::String),
                format: "date-time".into(),
                ..Default::default()
            }
        }
    }
}
