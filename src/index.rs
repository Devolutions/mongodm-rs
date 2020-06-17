//! Indexes are used for efficient mongo queries.

use mongodb::bson::doc;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use std::borrow::Cow;

/// Index sort order (useful for compound indexes).
///
/// [Mongo manual](https://docs.mongodb.com/manual/core/index-compound/#sort-order)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl From<SortOrder> for Bson {
    fn from(v: SortOrder) -> Self {
        match v {
            SortOrder::Ascending => Self::Int32(1),
            SortOrder::Descending => Self::Int32(-1),
        }
    }
}

#[derive(Debug, Clone)]
struct IndexKey {
    name: Cow<'static, str>,
    direction: SortOrder,
}

/// Specify field to be used for indexing and options.
///
/// [Mongo manual](https://docs.mongodb.com/manual/indexes/)
#[derive(Default, Clone, Debug)]
pub struct Index {
    keys: Vec<IndexKey>,
    options: Vec<IndexOption>,
}

impl Index {
    /// Make a new index for the given key with ascending direction.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/core/index-single/)
    pub fn new(key: impl Into<Cow<'static, str>>) -> Self {
        Self::new_with_direction(key, SortOrder::Ascending)
    }

    /// Make a new index for the given key with a direction.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/core/index-single/)
    pub fn new_with_direction(key: impl Into<Cow<'static, str>>, direction: SortOrder) -> Self {
        let mut index = Self::default();
        index.add_key_with_direction(key, direction);
        index
    }

    /// Make this index compound adding the given key with ascending direction.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/core/index-compound/).
    pub fn add_key(&mut self, key: impl Into<Cow<'static, str>>) {
        self.add_key_with_direction(key, SortOrder::Ascending)
    }

    /// Builder style method for `add_key`.
    pub fn with_key(mut self, key: impl Into<Cow<'static, str>>) -> Self {
        self.add_key(key);
        self
    }

    /// Make this index compound adding the given key with a direction.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/core/index-compound/).
    pub fn add_key_with_direction(
        &mut self,
        key: impl Into<Cow<'static, str>>,
        direction: SortOrder,
    ) {
        self.keys.push(IndexKey {
            name: key.into(),
            direction,
        });
    }

    /// Builder style method for `add_key_with_direction`.
    pub fn with_key_with_direction(
        mut self,
        key: impl Into<Cow<'static, str>>,
        direction: SortOrder,
    ) -> Self {
        self.add_key_with_direction(key, direction);
        self
    }

    /// Add an option to this index.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/reference/method/db.collection.createIndex/#options)
    pub fn add_option(&mut self, option: IndexOption) {
        self.options.push(option);
    }

    /// Builder style method for `add_option`.
    pub fn with_option(mut self, option: IndexOption) -> Self {
        self.add_option(option);
        self
    }

    /// Convert this structure into a `Document` version structured as expected by mongo.
    pub fn into_document(self) -> Document {
        // If document is missing "name" we follow default name generation as described in mongodb doc and
        // add it.
        // https://docs.mongodb.com/manual/indexes/#index-names
        // > The default name for an index is the concatenation of the
        // > indexed keys and each key’s direction in the index ( i.e. 1 or -1)
        // > using underscores as a separator.

        let mut names = Vec::with_capacity(self.keys.len());
        let mut keys_doc = Document::new();
        for key in self.keys {
            let key_name = match key.direction {
                SortOrder::Ascending => format!("{}_1", key.name),
                SortOrder::Descending => format!("{}_-1", key.name),
            };
            names.push(key_name);

            keys_doc.insert(key.name, key.direction);
        }

        let mut index_doc = doc! { "key": keys_doc };

        for option in self.options {
            let (key, value) = option.into_key_value();
            index_doc.insert(key, value);
        }

        if !index_doc.contains_key("name") {
            let name = names.join("_");
            index_doc.insert("name", name);
        }

        index_doc
    }
}

/// Collection of indexes. Provides function to build database commands.
///
/// [Mongo manual](https://docs.mongodb.com/manual/indexes/)
#[derive(Debug, Clone)]
pub struct Indexes(pub(crate) Vec<Index>);

impl Default for Indexes {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Index>> for Indexes {
    fn from(indexes: Vec<Index>) -> Self {
        Self(indexes)
    }
}

impl Indexes {
    /// New empty index list.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Builder style method to add an index.
    pub fn with(mut self, index: Index) -> Self {
        self.0.push(index);
        self
    }

    /// Generate `createIndexes` command document to submit to `Database::run_command`.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/reference/command/createIndexes/)
    pub fn create_indexes_command(self, collection_name: &str) -> Document {
        let mut indexes = Vec::with_capacity(self.0.len());
        for index in self.0 {
            indexes.push(index.into_document());
        }

        doc! {
            "createIndexes": collection_name,
            "indexes": indexes
        }
    }
}

/// Option to be used at index creation.
///
/// [Mongo manual](https://docs.mongodb.com/manual/reference/method/db.collection.createIndex/#options)
#[derive(Debug, Clone)]
pub enum IndexOption {
    /// Enable background builds
    Background,
    /// Creates a unique index
    Unique,
    /// Name of the index
    Name(String),
    /// Only references documents that match the filter expression
    PartialFilterExpression(Document),
    /// Only references documents with the specified field
    Sparse,
    /// TTL to control how long data is retained in the collectino
    ExpireAfterSeconds(i32),
    /// Configure the storage engine
    StorageEngine(Document),
    /// Specifies the collation
    Collation(Document),
    /// Specify a custom index option. This is present to provide forwards compatibility.
    Custom { name: String, value: Bson },
}

impl IndexOption {
    pub fn name(&self) -> &str {
        match self {
            IndexOption::Background => "background",
            IndexOption::Unique => "unique",
            IndexOption::Name(..) => "name",
            IndexOption::PartialFilterExpression(..) => "partialFilterExpression",
            IndexOption::Sparse => "sparse",
            IndexOption::ExpireAfterSeconds(..) => "expireAfterSeconds",
            IndexOption::StorageEngine(..) => "storageEngine",
            IndexOption::Collation(..) => "collation",
            IndexOption::Custom { name, .. } => name.as_str(),
        }
    }

    pub fn into_value(self) -> Bson {
        match self {
            IndexOption::Background | IndexOption::Unique | IndexOption::Sparse => {
                Bson::Boolean(true)
            }
            IndexOption::Name(val) => Bson::String(val),
            IndexOption::ExpireAfterSeconds(val) => Bson::Int32(val),
            IndexOption::PartialFilterExpression(doc)
            | IndexOption::StorageEngine(doc)
            | IndexOption::Collation(doc) => Bson::Document(doc),
            IndexOption::Custom { value, .. } => value,
        }
    }

    pub fn into_key_value(self) -> (String, Bson) {
        let name = self.name().to_owned();
        let value = self.into_value();
        (name, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_indexes_command() {
        let index = Index::new_with_direction("id", SortOrder::Descending)
            .with_key("last_seen")
            .with_option(IndexOption::Background)
            .with_option(IndexOption::Unique);

        let index_2 = Index::new("last_seen").with_option(IndexOption::ExpireAfterSeconds(60));

        let indexes = Indexes::from(vec![index, index_2]);

        assert_eq!(
            indexes.create_indexes_command("my_collection"),
            doc! {
                "createIndexes": "my_collection",
                "indexes": [
                    {
                        "key": { "id": -1, "last_seen": 1 },
                        "background": true,
                        "unique": true,
                        "name": "id_-1_last_seen_1",
                    },
                    {
                        "key": { "last_seen": 1 },
                        "expireAfterSeconds": 60,
                        "name": "last_seen_1",
                    },
                ]
            }
        );
    }
}
