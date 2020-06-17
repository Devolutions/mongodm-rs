//! Static operators for queries to prevent invalid queries due to typos.
//!
//! See mongo manual for [query operators](https://docs.mongodb.com/manual/reference/operator/query/)
//! and [update operators](https://docs.mongodb.com/manual/reference/operator/update/).
//!
//! If an operator is missing, you can easily add it yourself (also, PR are welcomed) or use the hardcoded
//! string like you would in a mongo shell.
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
    ($ty:ident => $mongo_operator:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[doc="Operator `"]
        #[doc=$mongo_operator]
        #[doc="`"]
        pub struct $ty;

        impl From<$ty> for ::std::string::String {
            fn from(_: $ty) -> ::std::string::String {
                ::std::string::String::from($mongo_operator)
            }
        }
    };
    ($category:literal [ $doc_url:literal ] : $ty:ident => $mongo_operator:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[doc="["]
        #[doc=$category]
        #[doc="]("]
        #[doc=$doc_url]
        #[doc=") "]
        #[doc="operator `"]
        #[doc=$mongo_operator]
        #[doc="`"]
        pub struct $ty;

        impl From<$ty> for ::std::string::String {
            fn from(_: $ty) -> ::std::string::String {
                ::std::string::String::from($mongo_operator)
            }
        }
    };
    ($category:literal [ $doc_url:literal ] : $( $ty:ident => $mongo_operator:literal, )+ ) => {
        $( declare_operator! { $category [ $doc_url ] : $ty => $mongo_operator } )+
    };
}

// == Query operators == //

declare_operator! { "Comparison" ["https://docs.mongodb.com/manual/reference/operator/query/#comparison"]:
    Equal => "$eq",
    GreaterThan => "$gt",
    GreaterThanEqual => "$gte",
    In => "$in",
    LesserThan => "$lt",
    LesserThanEqual => "$lte",
    NotEqual => "$ne",
    NoneIn => "$nin",
}

declare_operator! { "Logical" ["https://docs.mongodb.com/manual/reference/operator/query/#logical"]:
    And => "$and",
    Not => "$not",
    Nor => "$nor",
    Or => "$or",
}

declare_operator! { "Element" ["https://docs.mongodb.com/manual/reference/operator/query/#element"]:
    Exists => "$exists",
    Type => "$type",
}

declare_operator! { "Evaluation" ["https://docs.mongodb.com/manual/reference/operator/query/#evaluation"]:
    Expr => "$expr",
    JsonSchema => "$jsonSchema",
    Mod => "$mod",
    Regex => "$regex",
    Text => "$text",
    Where => "$where",
}

declare_operator! { "Geospatial" ["https://docs.mongodb.com/manual/reference/operator/query/#geospatial"]:
    GeoIntersects => "$geoIntersects",
    GeoWithin => "$geoWithin",
    Near => "$near",
    NearSphere => "$nearSphere",
}

declare_operator! { "Array (query)" ["https://docs.mongodb.com/manual/reference/operator/query/#array"]:
    All => "$all",
    ElemMatch => "$elemMatch",
    Size => "$size",
}

declare_operator! { "Bitwise (query)" ["https://docs.mongodb.com/manual/reference/operator/query/#bitwise"]:
    BitsAllClear => "$bitsAllClear",
    BitsAllSet => "$bitsAllSet",
    BitsAnyClear => "$bitsAnyClear",
    BitsAnySet => "$bitsAnySet",
}

declare_operator! { "Comments" ["https://docs.mongodb.com/manual/reference/operator/query/#comments"]: Comment => "$comment" }

declare_operator! { "Projection" ["https://docs.mongodb.com/manual/reference/operator/query/#projection-operators"]:
    ProjectFirst => "$",
    Meta => "$meta",
    Slice => "$slice",
}

// == Update operators ==

declare_operator! { "Fields" ["https://docs.mongodb.com/manual/reference/operator/update/#fields"]:
    CurrentDate => "$currentDate",
    Inc => "$inc",
    Min => "$min",
    Max => "$max",
    Mul => "$mul",
    Rename => "$rename",
    Set => "$set",
    SetOnInsert => "$setOnInsert",
    Unset => "$unset",
}

declare_operator! { "Array (update)" ["https://docs.mongodb.com/manual/reference/operator/update/#array"]:
    UpdateFirstDocument => "$",
    UpdateAllDocuments => "$[]",
    AddToSet => "$addToSet",
    Pop => "$pop",
    Pull => "$pull",
    Push => "$push",
    PullAll => "$pullAll",
}

declare_operator! { "Modifiers" ["https://docs.mongodb.com/manual/reference/operator/update/#modifiers"]:
    Each => "$each",
    Position => "$position",
    Sort => "$sort",
}

declare_operator! { "Bitwise (update)" ["https://docs.mongodb.com/manual/reference/operator/update/#bitwise"]:
    Bit => "$bit",
}
