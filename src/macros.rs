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
///# doc! { field!((bar in MyModel).(third in Bar).@@(b in Third)): 0 };
/// ```
/*
#[macro_export]
macro_rules! field {
    ( $field:ident in $type:path ) => {{
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn() = || {
            let $type { $field: _, .. };
        };
        stringify!( $field )
    }};
    ( @ $field:ident in $type:path ) => {{
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn() = || {
            let $type { $field: _, .. };
        };
        concat!( "$", stringify!($field) )
    }};
    ( @ @ $field:ident in $type:path ) => {{
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn() = || {
            let $type { $field: _, .. };
        };
        concat!( "$$", stringify!( $field ) )
    }};
    ( ( $field:ident in $type:path ) )  => {{
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn() = || {
            let $type { $field: _, .. };
        };
        stringify!( $field )
    }};
    // FIXME: ideally we want a compile-time string instead of format! (causing heap allocation,
    //        and returning a String instead of string literal)
    ( ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) $( . $rest:tt )* ) => {{
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn($type) = |a: $type| {
            let takes_type2 = |_: $type2| {};
            takes_type2(a.$field);
        };
        format!( "{}.{}", $crate::field!( $field in $type ), $crate::field!( ( $field2 in $type2 ) $( . $rest )* ) )
    }};
    ( @ ( $field:ident in $type:path ) . $( $rest:tt ).+ ) => {{
        format!( "${}", $crate::field!( ( $field in $type ) . $( $rest ).+ ) )
    }};
    ( @ @ ( $field:ident in $type:path ) . $( $rest:tt ).+ ) => {{
        format!( "$${}", $crate::field!( ( $field in $type ) . $( $rest ).+ ) )
    }};
}
*/
#[macro_export]
macro_rules! field {
    ( @string $field:ident in $type:path ) => {
        stringify!($field)
    };
    ( @string @ $field:ident in $type:path ) => {
        concat!( "$", stringify!($field) )
    };
    ( @string @ @ $field:ident in $type:path ) => {
        concat!( "$$", stringify!( $field ) )
    };
    ( @string ( $field:ident in $type:path ) ) => {
        stringify!($field)
    };
    ( @string @ ( $field:ident in $type:path ) ) => {
        concat!( "$", stringify!($field) )
    };
    ( @string @ @ ( $field:ident in $type:path ) ) => {
        concat!( "$$", stringify!( $field ) )
    };
    ( @string ( $field:ident in $type:path ) $( . $rest:tt )+ ) => {
        concat!( stringify!($field), ".", $crate::field!( @string $($rest).+ ) )
    };
    ( @string @ ( $field:ident in $type:path ) $( . $rest:tt )+ ) => {
        concat!( "$", stringify!($field), ".", $crate::field!( @string $($rest).+ ) )
    };
    ( @string @ @ ( $field:ident in $type:path ) $( . $rest:tt )+ ) => {
        concat!( "$$", stringify!($field), ".", $crate::field!( @string $($rest).+ ) )
    };
    
    ( @check $field:ident in $type:path ) => {
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn() = || {
            let $type { $field: _, .. };
        };
    };
    ( @check @ $field:ident in $type:path ) => { $crate::field!(@check $field in $type) };
    ( @check @ @ $field:ident in $type:path ) => { $crate::field!(@check $field in $type) };
    ( @check ( $field:ident in $type:path ) ) => { $crate::field!(@check $field in $type) };
    ( @check @ ( $field:ident in $type:path ) ) => { $crate::field!(@check $field in $type) };
    ( @check @ @ ( $field:ident in $type:path ) ) => { $crate::field!(@check $field in $type) };
    ( @check ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) ) => {
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn($type) = |a: $type| {
            let takes_type2 = |_: $type2| {};
            takes_type2(a.$field);
        };
        $crate::field!(@check $field in $type);
        $crate::field!(@check $field2 in $type2);
    };
    ( @check ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) . $($rest:tt)+ ) => {
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn($type) = |a: $type| {
            let takes_type2 = |_: $type2| {};
            takes_type2(a.$field);
        };
        $crate::field!(@check $field in $type);
        $crate::field!(@check ( $field2 in $type2 ) . $($rest)+)
    };
    ( @check @ ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) ) => {
        $crate::field!(@check ( $field in $type ) . ( $field2 in $type2 ) )
    };
    ( @check @ ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) . $($rest:tt)+ ) => {
        $crate::field!(@check ( $field in $type ) . ( $field2 in $type2 ) . $($rest)+ )
    };
    ( @check @ @ ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) ) => {
        $crate::field!(@check ( $field in $type ) . ( $field2 in $type2 ) )
    };
    ( @check @ @ ( $field:ident in $type:path ) . ( $field2:ident in $type2:path ) . $($rest:tt)+ ) => {
        $crate::field!(@check ( $field in $type ) . ( $field2 in $type2 ) . $($rest)+ )
    };

    ( $($rest:tt)* ) => {{
        $crate::field! { @check $($rest)* }
        $crate::field! { @string $($rest)* }
    }};
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
    ( $( $tt:tt )* ) => { $crate::field! { $( $tt )* } }
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
    // Last key-value with trailing comma
    (@stages $vec:ident $key:ident : $value:tt ,) => {
        $vec.push($crate::mongo::bson::doc! { $key : $value });
    };

    // Last key-value without trailing comma
    (@stages $vec:ident $key:ident : $value:tt) => {
        $vec.push($crate::mongo::bson::doc! { $key : $value });
    };

    // key-value + rest
    (@stages $vec:ident $key:ident : $value:tt , $($rest:tt)*) => {{
        $vec.push($crate::mongo::bson::doc! { $key : $value });
        pipeline!(@stages $vec $($rest)*);
    }};

    // Last expr with trailing comma
    (@stages $vec:ident $stage:expr ,) => {
        pipeline!(@stage $vec $stage);
    };

    // Last expr without trailing comma
    (@stages $vec:ident $stage:expr) => {
        pipeline!(@stage $vec $stage);
    };

    // expr + rest
    (@stages $vec:ident $stage:expr , $($rest:tt)*) => {{
        pipeline!(@stage $vec $stage);
        pipeline!(@stages $vec $($rest)*);
    }};

    (@stage $vec:ident $stage:expr ) => {
        $vec.push($crate::mongo::bson::Document::from($stage));
    };

    ($($tt:tt)*)=> {{
        let mut vec = vec![];
        pipeline!(@stages vec $($tt)*);
        vec
    }};
}