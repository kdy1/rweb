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

#[derive(Debug)]
pub struct ComponentDescriptor {
    components: IndexMap<Cow<'static, str>, Schema>,
}
impl ComponentDescriptor {
    pub(crate) fn new() -> Self {
        Self {
            components: IndexMap::new(),
        }
    }
    /// Get a reference to the component named `name`, if such exists.
    pub fn get_component(&self, name: &str) -> Option<&Schema> {
        self.components.get(name)
    }
    /// Get a reference to the schema of a type.
    ///
    /// If `schema` is inline, it itself is returned,
    /// otherwise the component is looked up by name and its schema is returned.
    ///
    /// # Panics
    /// Panics if `schema` refers to a non-existing component.
    pub fn get_unpack<'a>(&'a self, schema: &'a ComponentOrInlineSchema) -> &'a Schema {
        match schema {
            ComponentOrInlineSchema::Component { name } => self.get_component(name).unwrap(),
            ComponentOrInlineSchema::Inline(s) => s,
        }
    }
    /// Describes a component, iff it isn't already described.
    ///
    /// # Parameters
    /// - `name`: name of the component
    /// - `desc`: descriptor function
    ///
    /// # Returns
    /// Reference to the component
    ///
    /// # Circular references
    /// To avoid infinite recursion on circular references,
    /// a blanket schema is stored under component name first,
    /// then the component is described and the schema replaced.
    ///
    /// Note that this _may_ cause invalid spec generation if
    /// somewhere in such loop there are types that rely on
    /// cloned modification of the schema of underlying component.
    pub fn describe_component(
        &mut self,
        name: &str,
        desc: impl FnOnce(&mut ComponentDescriptor) -> Schema,
    ) -> ComponentOrInlineSchema {
        if !self.components.contains_key(name) {
            self.components
                .insert(Cow::Owned(name.to_string()), Default::default());
            self.components[name] = desc(self);
        }
        ComponentOrInlineSchema::Component {
            name: Cow::Owned(name.to_string()),
        }
    }
    /// Finalizes the descriptor and packages up all components.
    pub(crate) fn build(self) -> IndexMap<Cow<'static, str>, ObjectOrReference<Schema>> {
        self.components
            .into_iter()
            .map(|(k, v)| (k, ObjectOrReference::Object(v)))
            .collect()
    }
}

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
    /// String uniquely identifying this type, respecting component naming pattern.
    ///
    /// If this type is a component, this is the component's name.
    ///
    /// Even if this type is not a component, this is necessary for assembling names of generic components
    /// parameterized on underlying types.
    ///
    /// # Returns
    /// Name of this type, respecting `^[a-zA-Z0-9\.\-_]+$` regex.
    ///
    /// # Panics
    /// Panic if you decide that this type must not be used for generic parameterization of components.
    fn type_name() -> Cow<'static, str>;

    /// Describe this entity, and the components it (may) requires.
    fn describe(comp_d: &mut ComponentDescriptor) -> ComponentOrInlineSchema;
}

/// This should be implemented only for types that know how it should be
/// encoded.
pub trait ResponseEntity: Entity {
    fn describe_responses() -> Responses;
}

/// Implements entity by another entity
macro_rules! delegate_entity {
	// full paths (with `::`) not supported
	( $T:tt $(< $( $tlt:tt $(< $( $tltt:tt ),+ >)? ),+ >)? => $D:tt $(< $( $plt:tt $(< $( $pltt:tt ),+ >)? ),+ >)? ) => {
		impl Entity for $T $(< $( $tlt $(< $( $tltt ),+ >)? ),+ >)? {
			fn type_name() -> Cow<'static, str> {
				<$D $(< $( $plt $(< $( $pltt ),+ >)? ),+ >)? as Entity>::type_name()
			}
			fn describe(d: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
				<$D $(< $( $plt $(< $( $pltt ),+ >)? ),+ >)? as Entity>::describe(d)
			}
		}
    };
	// Doesn't work with `?Sized` :(
	( < $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ > $T:tt $(< $( $tlt:tt $(< $( $tltt:tt ),+ >)? ),+ >)? => $D:tt $(< $( $plt:tt $(< $( $pltt:tt ),+ >)? ),+ >)? ) => {
        impl < $( $lt $( : $clt $(+ $dlt )* )? ),+ > Entity for $T $(< $( $tlt $(< $( $tltt ),+ >)? ),+ >)? {
            fn type_name() -> Cow<'static, str> {
                <$D $(< $( $plt $(< $( $pltt ),+ >)? ),+ >)? as Entity>::type_name()
            }
            fn describe(d: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
                <$D $(< $( $plt $(< $( $pltt ),+ >)? ),+ >)? as Entity>::describe(d)
            }
        }
    };
}

impl Entity for () {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("unit")
    }
    /// Returns empty schema
    #[inline]
    fn describe(_: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
        ComponentOrInlineSchema::Inline(Schema {
            schema_type: Some(Type::Object),
            ..Default::default()
        })
    }
}

macro_rules! integer {
    ($T:ty) => {
        impl Entity for $T {
			fn type_name() -> Cow<'static, str> {
				Cow::Borrowed("int")
			}
            #[inline]
            fn describe(_: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
                ComponentOrInlineSchema::Inline(Schema {
                    schema_type: Some(Type::Integer),
                    ..Default::default()
                })
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
            fn type_name() -> Cow<'static, str> {
                Cow::Borrowed("number")
            }
            #[inline]
            fn describe(_: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
                ComponentOrInlineSchema::Inline(Schema {
                    schema_type: Some(Type::Number),
                    ..Default::default()
                })
            }
        }
    };
}

number!(f32);
number!(f64);

impl Entity for bool {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("bool")
    }
    #[inline]
    fn describe(_: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
        ComponentOrInlineSchema::Inline(Schema {
            schema_type: Some(Type::Boolean),
            ..Default::default()
        })
    }
}

impl Entity for char {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("char")
    }
    #[inline]
    fn describe(_: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
        ComponentOrInlineSchema::Inline(Schema {
            schema_type: Some(Type::String),
            ..Default::default()
        })
    }
}

impl Entity for str {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("string")
    }
    #[inline]
    fn describe(_: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
        ComponentOrInlineSchema::Inline(Schema {
            schema_type: Some(Type::String),
            ..Default::default()
        })
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
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn describe(comp_d: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
        T::describe(comp_d)
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
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn describe(comp_d: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
        T::describe(comp_d)
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
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn describe(comp_d: &mut ComponentDescriptor) -> ComponentOrInlineSchema {
        T::describe(comp_d)
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

delegate_entity!(String => str);

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
                ObjectOrReference::Object(Schema {
                    schema_type: Some(Type::Object),
                    properties: indexmap::indexmap! {
                        Cow::Borrowed("Ok") => T::describe(),
                    },
                    required: vec![Cow::Borrowed("Ok")],
                    ..Default::default()
                }),
                ObjectOrReference::Object(Schema {
                    schema_type: Some(Type::Object),
                    properties: indexmap::indexmap! {
                        Cow::Borrowed("Err") => E::describe(),
                    },
                    required: vec![Cow::Borrowed("Err")],
                    ..Default::default()
                }),
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

delegate_entity!(<V: Entity, S> HashSet<V, S> => BTreeSet<V>);

delegate_entity!(<T: Entity> Vec<T> => [T]);
delegate_entity!(<T: Entity> LinkedList<T> => [T]);
delegate_entity!(<T: Entity> VecDeque<T> => [T]);

delegate_entity!(<T: Entity> (T, T) => [T; 2]);
delegate_entity!(<T: Entity> (T, T, T) => [T; 3]);
delegate_entity!(<T: Entity> (T, T, T, T) => [T; 4]);
delegate_entity!(<T: Entity> (T, T, T, T, T) => [T; 5]);

delegate_entity!(<T: Entity> HashMap<Arc<String>, T> => HashMap<String, T>);
delegate_entity!(<T: Entity> HashMap<Cow<'_, String>, T> => HashMap<String, T>);

delegate_entity!(<T: Entity> BTreeMap<String, T> => HashMap<String, T>);
delegate_entity!(<T: Entity> BTreeMap<Arc<String>, T> => BTreeMap<String, T>);
delegate_entity!(<T: Entity> BTreeMap<Cow<'_, String>, T> => BTreeMap<String, T>);

delegate_entity!(<T: Entity> IndexMap<String, T> => HashMap<String, T>);
delegate_entity!(<T: Entity> IndexMap<Arc<String>, T> => IndexMap<String, T>);
delegate_entity!(<T: Entity> IndexMap<Cow<'_, String>, T> => IndexMap<String, T>);

delegate_entity!(Infallible => ());

impl ResponseEntity for Infallible {
    #[inline]
    fn describe_responses() -> Responses {
        Default::default()
    }
}

delegate_entity!(<T: Entity> Json<T> => T);

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

type SerdeJsonValue = serde_json::Value;
delegate_entity!(SerdeJsonValue => ());

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

delegate_entity!(<T: Entity> Query<T> => T);
delegate_entity!(<T: Entity> Form<T> => T);

delegate_entity!(Rejection => ());

impl ResponseEntity for Rejection {
    fn describe_responses() -> Responses {
        Default::default()
    }
}

type HttpError = http::Error;
delegate_entity!(HttpError => ());

impl ResponseEntity for http::Error {
    fn describe_responses() -> Responses {
        Default::default()
    }
}

type DynReply = dyn Reply;
delegate_entity!(DynReply => ());

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
