//! Static operators for queries to prevent invalid queries due to typos
//!
//! See [mongo manual](https://docs.mongodb.com/manual/reference/operator/query/).
//!
//! ```
//! use mongodm::mongo::bson::doc;
//! use mongodm::operator::*;
//!
//! // Using static operators
//! let a = doc! {
//!     And: [
//!         { "foo": { Exists: true } },
//!         {
//!             Or: [
//!                 { "bar": { GreaterThan: 100 } },
//!                 { "lorem": "ipsum" }
//!             ]
//!         }
//!     ]
//! };
//!
//! // Using hardcoded strings
//! let b = doc! {
//!     "$and": [
//!         { "foo": { "$exists": true } },
//!         {
//!             "$or": [
//!                 { "bar": { "$gt": 100 } },
//!                 { "lorem": "ipsum" }
//!             ]
//!         }
//!     ]
//! };
//!
//! // Generated document are identicals
//! assert_eq!(a, b);
//! ```

macro_rules! declare_operator {
    ($ty:ident, $mongo_operator:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $ty;

        impl From<$ty> for ::std::string::String {
            fn from(_: $ty) -> ::std::string::String {
                ::std::string::String::from($mongo_operator)
            }
        }
    };
}

// Comparison

declare_operator! { Equal, "$eq" }
declare_operator! { GreaterThan, "$gt" }
declare_operator! { GreaterThanEqual, "$gte" }
declare_operator! { In, "$in" }
declare_operator! { LesserThan, "$lt" }
declare_operator! { LesserThanEqual, "$lte" }
declare_operator! { NotEqual, "$ne" }
declare_operator! { NoneIn, "$nin" }

// Logical

declare_operator! { And, "$and" }
declare_operator! { Not, "$not" }
declare_operator! { Nor, "$nor" }
declare_operator! { Or, "$or" }

// Element

declare_operator! { Exists, "$exists" }
declare_operator! { Type, "$type" }

// Evaluation

declare_operator! { Expr, "$expr" }
declare_operator! { JsonSchema, "$jsonSchema" }
declare_operator! { Mod, "$mod" }
declare_operator! { Regex, "$regex" }
declare_operator! { Text, "$text" }
declare_operator! { Where, "$where" }

// Geospatial

declare_operator! { GeoIntersects, "$geoIntersects" }
declare_operator! { GeoWithin, "$geoWithin" }
declare_operator! { Near, "$near" }
declare_operator! { NearSphere, "$nearSphere" }

// Array

declare_operator! { All, "$all" }
declare_operator! { ElemMatch, "$elemMatch" }
declare_operator! { Size, "$size" }

// Bitwise

declare_operator! { BitsAllClear, "$bitsAllClear" }
declare_operator! { BitsAllSet, "$bitsAllSet" }
declare_operator! { BitsAnyClear, "$bitsAnyClear" }
declare_operator! { BitsAnySet, "$bitsAnySet" }

// Comments

declare_operator! { Comment, "$comment" }

// Projection operations

declare_operator! { Meta, "$meta" }
declare_operator! { Slice, "$slice" }
