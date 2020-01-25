use crate::{Form, Json, Query};
pub use rweb_openapi::v3_0::*;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    convert::Infallible,
};
use warp::{Rejection, Reply};

pub type Components = Vec<(Cow<'static, str>, Schema)>;

pub type Responses = BTreeMap<Cow<'static, str>, Response>;

/// This can be derived by `#[derive(Schema)]`.
///
///
/// You may provide an example value of each field with
///
/// `#[schema(example = "path_to_function")]`
pub trait Entity {
    fn describe() -> Schema;

    fn describe_components() -> Components {
        Default::default()
    }
}

/// THis should be implemented only for types that know how it should be
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
            schema_type: Type::Object,
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

macro_rules! number {
    ($T:ty) => {
        impl Entity for $T {
            #[inline]
            fn describe() -> Schema {
                Schema {
                    schema_type: Type::Number,
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
            schema_type: Type::Boolean,
            ..Default::default()
        }
    }
}

impl Entity for char {
    #[inline]
    fn describe() -> Schema {
        Schema {
            schema_type: Type::String,
            ..Default::default()
        }
    }
}

impl Entity for str {
    #[inline]
    fn describe() -> Schema {
        Schema {
            schema_type: Type::String,
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

impl<T: Entity> Entity for [T] {
    fn describe() -> Schema {
        Schema {
            schema_type: Type::Array,
            items: Some(Box::new(T::describe())),
            ..Default::default()
        }
    }

    fn describe_components() -> Components {
        T::describe_components()
            .into_iter()
            .map(|(name, s)| {
                (
                    Cow::Owned(format!("{}List", name)),
                    Schema {
                        schema_type: Type::Array,
                        items: Some(Box::new(s)),
                        ..Default::default()
                    },
                )
            })
            .collect()
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

    fn describe_components() -> Components {
        let mut v = T::describe_components();
        for (_, s) in v.iter_mut() {
            s.nullable = Some(true);
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
        let mut content = BTreeMap::new();
        content.insert(
            Cow::Borrowed("text/plain"),
            MediaType {
                schema: Some(ObjectOrReference::Object(Self::describe())),
                examples: None,
                encoding: Default::default(),
            },
        );

        let mut map = BTreeMap::new();
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
    fn describe_responses() -> BTreeMap<Cow<'static, str>, Response> {
        let mut map = T::describe_responses();
        map.extend(E::describe_responses());
        map
    }
}

//impl<K, V> Entity for BTreeMap<K, V> {}

impl<V> Entity for BTreeSet<V>
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

//impl<V> Entity for BinaryHeap<V> {}

//impl<K, V, S> Entity for HashMap<K, V, S> {}

impl<V, S> Entity for HashSet<V, S>
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
        let mut content = BTreeMap::new();
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
