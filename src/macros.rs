/// Statically check presence of field in a given struct and stringify it.
///
/// Note that it sadly won't work with `#[serde(rename = "...")]` and `#[serde(rename_all = "...")]`.
///
/// # Example
///
/// ```
/// use mongodm::mongo::bson::doc;
/// use mongodm::field;
/// use mongodm::operator::*;
///
/// struct MyModel {
///     foo: i64,
///     bar: i64,
///     lorem: String,
/// }
///
/// // Statically checked
/// let a = doc! {
///     And: [
///         { field!(foo in MyModel): { Exists: true } },
///         {
///             Or: [
///                 { field!(bar in MyModel): { GreaterThan: 100 } },
///                 { field!(lorem in MyModel): "ipsum" }
///             ]
///         },
///         // dollar and double dollar signs can inserted by prefixing with @
///         { field!(@foo in MyModel): field!(@@bar in MyModel) }
///     ]
/// };
///
/// // Hardcoded strings
/// let b = doc! {
///     "$and": [
///         { "foo": { "$exists": true } },
///         {
///             "$or": [
///                 { "bar": { "$gt": 100 } },
///                 { "lorem": "ipsum" }
///             ]
///         },
///         { "$foo": "$$bar" }
///     ]
/// };
///
/// // Generated document are identicals
/// assert_eq!(a, b);
/// ```
///
/// Nested structs
/// ```
/// use mongodm::mongo::bson::doc;
/// use mongodm::field;
///
/// struct Foo {
///     bar: Bar,
/// }
///
/// struct Bar {
///     lorem: String,
///     baz: Baz,
/// }
///
/// struct Baz {
///     dolor: String,
/// }
///
/// assert_eq!(
///     doc! { field!((bar in Foo).(lorem in Bar)): "ipsum" },
///     doc! { "bar.lorem": "ipsum" },
/// );
///
/// assert_eq!(
///     doc! { field!((bar in Foo).(baz in Bar).(dolor in Baz)): "sit amet" },
///     doc! { "bar.baz.dolor": "sit amet" },
/// );
///
/// assert_eq!(
///     doc! { field!(@@(bar in Foo).(baz in Bar).(dolor in Baz)): "sit amet" },
///     doc! { "$$bar.baz.dolor": "sit amet" },
/// );
/// ```
///
/// If the field doesn't exist, compilation will fail.
///
/// ```compile_fail
///# use mongodm::mongo::bson::doc;
///# use mongodm::field;
///#
/// struct MyModel {
///     foo: i64,
///     bar: Bar,
/// }
///
/// struct Bar {
///     baz: i64,
/// }
///
/// // Doesn't compile because `baz` isn't a member of `MyModel`
/// doc! { field!(baz in MyModel): 0 };
/// ```
///
/// ```compile_fail
///# use mongodm::mongo::bson::doc;
///# use mongodm::field;
///#
///# struct MyModel {
///#     foo: i64,
///#     bar: Bar,
///# }
///#
///# struct Bar {
///#     baz: i64,
///# }
///#
/// // Doesn't compile because `foo` isn't a member of `Bar`
/// doc! { field!((bar in MyModel).(foo in Bar)): 0 };
/// ```
///
/// ```compile_fail
///# use mongodm::mongo::bson::doc;
///# use mongodm::field;
///#
///# struct MyModel {
///#     foo: i64,
///#     bar: Bar,
///# }
///#
///# struct Bar {
///#     baz: i64,
///# }
///#
/// // Doesn't compile because `foo` isn't a `Bar`
/// doc! { field!((foo in MyModel).(baz in Bar)): 0 };
/// ```
/// ```compile_fail
///# use mongodm::mongo::bson::doc;
///# use mongodm::field;
///#
///# struct MyModel {
///#     foo: i64,
///#     bar: Bar,
///# }
///#
///# struct Bar {
///#     baz: i64,
///#     third: Third,
///# }
///#
///# struct Third {
///#     a: String,
///# }
///#
/// // Fail because `b` is not a field of `Third`
/// doc! { field!((bar in MyModel).(third in Bar).(b in Third)): 0 };
/// ```
#[macro_export]
macro_rules! field {
    ( $($tt:tt)* ) => {{
        $crate::field_check_helper! { $($tt)* }
        $crate::field_string_helper! { $($tt)* }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! field_string_helper {
    ( $field:ident in $type:path ) => {
        stringify!($field)
    };
    ( @ $field:ident in $type:path ) => {
        concat!( "$", stringify!($field) )
    };
    ( @ @ $field:ident in $type:path ) => {
        concat!( "$$", stringify!( $field ) )
    };
    ( ( $field:ident in $type:path ) ) => {
        stringify!($field)
    };
    ( ( $field:ident in $type:path ) $( . $rest:tt )+ ) => {
        concat!( stringify!($field), ".", $crate::field_string_helper!($($rest).+) )
    };
    ( @ ( $field:ident in $type:path ) $( . $rest:tt )+ ) => {
        concat!( "$", stringify!($field), ".", $crate::field_string_helper!($($rest).+) )
    };
    ( @ @ ( $field:ident in $type:path ) $( . $rest:tt )+ ) => {
        concat!( "$$", stringify!($field), ".", $crate::field_string_helper!($($rest).+) )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! field_check_helper {
    ( $field:ident in $type:path ) => {
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn() = || {
            let $type { $field: _, .. };
        };
    };
    ( @ $field:ident in $type:path ) => { $crate::field_check_helper!($field in $type) };
    ( @ @ $field:ident in $type:path ) => { $crate::field_check_helper!($field in $type) };
    ( ( $field:ident in $type:path ) ) => { $crate::field_check_helper!($field in $type) };
    ( @ ( $field:ident in $type:path ) ) => { $crate::field_check_helper!($field in $type) };
    ( @ @ ( $field:ident in $type:path ) ) => { $crate::field_check_helper!($field in $type) };
    ( ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) ) => {
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn($type) = |a: $type| {
            let takes_type2 = |_: $type2| {};
            takes_type2(a.$field);
        };
        $crate::field_check_helper!($field in $type);
        $crate::field_check_helper!($field2 in $type2);
    };
    ( ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) . $($rest:tt)+ ) => {
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn($type) = |a: $type| {
            let takes_type2 = |_: $type2| {};
            takes_type2(a.$field);
        };
        $crate::field_check_helper!($field in $type);
        $crate::field_check_helper!(( $field2 in $type2 ) . $($rest)+)
    };
    ( @ ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) ) => {
        $crate::field_check_helper!(( $field in $type ) . ( $field2 in $type2 ) )
    };
    ( @ ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) . $($rest:tt)+ ) => {
        $crate::field_check_helper!(( $field in $type ) . ( $field2 in $type2 ) . $($rest)+ )
    };
    ( @ @ ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) ) => {
        $crate::field_check_helper!(( $field in $type ) . ( $field2 in $type2 ) )
    };
    ( @ @ ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) . $($rest:tt)+ ) => {
        $crate::field_check_helper!(( $field in $type ) . ( $field2 in $type2 ) . $($rest)+ )
    };
    // FIXME: Add rules to allow nesting Vec<> and Option<>
}

/// Shorthand for `field!`.
///
/// # Example
///
/// ```
/// use mongodm::mongo::bson::doc;
/// use mongodm::f;
/// use mongodm::operator::*;
///
/// struct MyModel {
///     foo: i64,
///     bar: i64,
///     lorem: String,
/// }
///
/// // Statically checked
/// let a = doc! {
///     And: [
///         { f!(foo in MyModel): { Exists: true } },
///         {
///             Or: [
///                 { f!(bar in MyModel): { GreaterThan: 100 } },
///                 { f!(lorem in MyModel): "ipsum" }
///             ]
///         },
///         // dollar and double dollar signs can inserted by prefixing with @
///         { f!(@foo in MyModel): f!(@@bar in MyModel) }
///     ]
/// };
///
/// // Hardcoded strings
/// let b = doc! {
///     "$and": [
///         { "foo": { "$exists": true } },
///         {
///             "$or": [
///                 { "bar": { "$gt": 100 } },
///                 { "lorem": "ipsum" }
///             ]
///         },
///         { "$foo": "$$bar" }
///     ]
/// };
///
/// // Generated document are identicals
/// assert_eq!(a, b);
/// ```
#[macro_export]
macro_rules! f {
    ( $($tt:tt)* ) => {{
        $crate::field_check_helper! { $($tt)* }
        $crate::field_string_helper! { $($tt)* }
    }};
}

/// Helper to build aggregation pipelines.
/// Return a Vec<Document> as expected by the aggregate function.
///
/// # Example
///
/// ```
/// use mongodm::prelude::*;
///
/// struct User {
///     _id: ObjectId,
///     name: String,
/// }
///
/// struct Session {
///     user_id: ObjectId,
/// }
///
/// // Using the pipeline! helper :
/// let a = pipeline! [
///     Match: { f!(name in User): "John" },
///     Lookup {
///         From: "sessions",
///         As: "sessions",
///         LocalField: f!(_id in User),
///         ForeignField: f!(user_id in Session),
///     }
/// ];
///
/// // Without pipeline helper :
/// let b = vec![
///     doc! { "$match": { f!(name in User): "John" } },
///     Lookup {
///         From: "sessions",
///         As: "sessions",
///         LocalField: f!(_id in User),
///         ForeignField: f!(user_id in Session),
///     }.into(),
/// ];
///
/// // Without any helpers :
/// let c = vec![
///     doc! { "$match": { "name": "John" } },
///     doc! { "$lookup": {
///         "from": "sessions",
///         "as": "sessions",
///         "localField": "_id",
///         "foreignField": "user_id",
///     } },
/// ];
///
/// // Generated pipelines are identicals
/// assert_eq!(a, b);
/// assert_eq!(a, c);
/// ```
#[macro_export]
macro_rules! pipeline {
    ($($tt:tt)*)=> {{
        let mut vec = vec![];
        $crate::pipeline_helper!(vec $($tt)*);
        vec
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! pipeline_helper {
    // Last key-value with trailing comma
    ($vec:ident $key:ident : $value:tt ,) => {
        $vec.push($crate::mongo::bson::doc! { $key : $value });
    };

    // Last key-value without trailing comma
    ($vec:ident $key:ident : $value:tt) => {
        $vec.push($crate::mongo::bson::doc! { $key : $value });
    };

    // key-value + rest
    ($vec:ident $key:ident : $value:tt , $($rest:tt)*) => {{
        $vec.push($crate::mongo::bson::doc! { $key : $value });
        $crate::pipeline_helper!($vec $($rest)*);
    }};

    // Last expr with trailing comma
    ($vec:ident $stage:expr ,) => {
        $vec.push($crate::mongo::bson::Document::from($stage));
    };

    // Last expr without trailing comma
    ($vec:ident $stage:expr) => {
        $vec.push($crate::mongo::bson::Document::from($stage));
    };

    // expr + rest
    ($vec:ident $stage:expr , $($rest:tt)*) => {{
        $vec.push($crate::mongo::bson::Document::from($stage));
        $crate::pipeline_helper!($vec $($rest)*);
    }};
}
