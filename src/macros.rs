/// Statically check presence of field in a given type and stringify it.
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
///         }
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
///         }
///     ]
/// };
///
/// // Generated document are identicals
/// assert_eq!(a, b);
/// ```
///
/// If the field doesn't exist, compilation will fail.
///
/// ```compile_fail
/// use mongodm::mongo::bson::doc;
/// use mongodm::field;
/// use mongodm::operator::*;
///
/// struct MyModel {
///     bar: i64,
/// }
///
/// // Doesn't compile because `foo` isn't a member of `MyModel`
/// doc! { field!(foo in MyModel): 0 };
/// ```
#[macro_export]
macro_rules! field {
    ( $field:ident in $type:path ) => {{
        #[allow(unknown_lints, unneeded_field_pattern)]
        const _: fn() = || {
            let $type { $field: _, .. };
        };
        stringify!($field)
    }};
}

/// Shorthand for `field!`
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
///         }
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
///         }
///     ]
/// };
///
/// // Generated document are identicals
/// assert_eq!(a, b);
/// ```
#[macro_export]
macro_rules! f {
    ( $field:ident in $type:path ) => {{
        $crate::field!($field in $type)
    }};
}
