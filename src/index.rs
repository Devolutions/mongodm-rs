//! Indexes are used for efficient mongo queries.

use crate::CollectionConfig;
use mongodb::bson::{doc, from_bson, Bson, Document};
use mongodb::options::*;
use mongodb::Database;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;

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

#[derive(Clone, Debug)]
enum IndexKey {
    SortIndex(SortIndexKey),
    TextIndex(TextIndexKey),
}

impl IndexKey {
    fn get_key_name(&self) -> String {
        match self {
            IndexKey::SortIndex(s) => match s.direction {
                SortOrder::Ascending => format!("{}_1", s.name),
                SortOrder::Descending => format!("{}_-1", s.name),
            },

            IndexKey::TextIndex(t) => format!("{}_text", t.name),
        }
    }

    fn get_name(&self) -> String {
        match self {
            IndexKey::SortIndex(s) => s.name.to_string(),
            IndexKey::TextIndex(t) => t.name.to_string(),
        }
    }

    fn get_value(&self) -> Bson {
        match self {
            IndexKey::SortIndex(s) => s.direction.into(),
            IndexKey::TextIndex(_) => "text".into(),
        }
    }
}

#[derive(Debug, Clone)]
struct SortIndexKey {
    name: Cow<'static, str>,
    direction: SortOrder,
}

#[derive(Debug, Clone)]
struct TextIndexKey {
    name: Cow<'static, str>,
}

/// Specify field to be used for indexing and options.
///
/// [Mongo manual](https://docs.mongodb.com/manual/indexes/)
///
/// # Example
/// ```
/// use mongodm::{Index, SortOrder, IndexOption, mongo::bson::doc};
///
/// let index = Index::new_with_direction("username", SortOrder::Descending)
///     .with_key("last_seen") // compound with last_seen
///     .with_option(IndexOption::Unique);
///
/// let doc = index.into_document();
///
/// assert_eq!(
///     doc,
///     doc! {
///         "key": { "username": -1, "last_seen": 1 },
///         "unique": true,
///         "name": "username_-1_last_seen_1",
///     }
/// )
/// ```
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

    /// Make a new index for the given key with the text parameter.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/core/index-single/)
    pub fn new_with_text(key: impl Into<Cow<'static, str>>) -> Self {
        let mut index = Self::default();
        index.add_key_with_text(key);
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
        self.keys.push(IndexKey::SortIndex(SortIndexKey {
            name: key.into(),
            direction,
        }));
    }

    /// Make this index compound adding the given key with text.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/core/index-compound/).
    pub fn add_key_with_text(&mut self, key: impl Into<Cow<'static, str>>) {
        self.keys
            .push(IndexKey::TextIndex(TextIndexKey { name: key.into() }));
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
            names.push(key.get_key_name());
            keys_doc.insert(key.get_name(), key.get_value());
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
    /// Specifies the weights for text indexes
    Weights(Vec<(String, i32)>),
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
            IndexOption::Weights(..) => "weights",
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
            IndexOption::Weights(w) => {
                let mut doc = Document::new();
                w.into_iter().for_each(|(k, v)| {
                    doc.insert(k, Bson::from(v));
                });
                Bson::Document(doc)
            }
            IndexOption::Custom { value, .. } => value,
        }
    }

    pub fn into_key_value(self) -> (String, Bson) {
        let name = self.name().to_owned();
        let value = self.into_value();
        (name, value)
    }
}

/// Synchronize backend mongo collection for a given `CollectionConfig`.
///
/// This should be called once per `CollectionConfig` on startup to synchronize indexes.
/// Indexes found in the backend and not defined in the model are destroyed except for the special index "_id".
pub async fn sync_indexes<CollConf: CollectionConfig>(
    db: &Database,
) -> Result<(), mongodb::error::Error> {
    let mut indexes = CollConf::indexes();

    match h_run_command(db, doc! { "listIndexes": CollConf::collection_name() }).await {
        Ok(ret) => {
            let parsed_ret: ListIndexesRet = from_bson(Bson::Document(ret))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            if parsed_ret.cursor.id != 0 {
                // batch isn't complete
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "couldn't list all indexes from '{}'",
                        CollConf::collection_name()
                    ),
                )
                .into());
            }

            let mut existing_indexes = HashMap::new();
            for index in parsed_ret.cursor.first_batch {
                if let Some(key) = index.get("key") {
                    existing_indexes.insert(key.to_string(), index);
                }
            }

            let mut already_sync = Vec::new();
            let mut to_drop = Vec::new();
            for (i, index) in indexes.0.clone().into_iter().enumerate() {
                let mut text_index_keys = None;
                let index_doc = if index.keys.iter().any(|ind| matches!(ind, IndexKey::TextIndex(_))) {
                    let mut doc = index.into_document();

                    // There an only be 1 text index per collection so when a text index is saved, the keys are automatically changed to this. We keep a copy for the weight comparison.
                    text_index_keys = doc.get("key").cloned();
                    doc.insert("key", doc! { "_fts": "text", "_ftsx": 1 });
                    doc
                } else {
                    index.into_document()
                };

                let key = index_doc.get("key").ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "index doc is missing 'key'")
                })?;
                if let Some(mut existing_index) = existing_indexes.remove(&key.to_string()) {
                    // "ns" and "v" in the response should not be used for the comparison
                    existing_index.remove("ns");
                    existing_index.remove("v");

                    // We compare the text index here, the keys become weights of 1 after saving in the DB. Custom weights not supported yet.
                    if let Some(Bson::Document(mut keys_to_set)) = text_index_keys {
                        if let Some(Bson::Document(existing_weights)) = existing_index.get("weights") {
                            // Changing all text values to the default weight of 1
                            for keys in keys_to_set.iter_mut() {
                                match keys.1 {
                                    Bson::String(t) if t == "text" => {
                                        *keys.1 = Bson::Int32(1);
                                    },
                                    _ => ()
                                }
                            }

                            if !existing_weights.eq(&keys_to_set) {
                                to_drop.push(
                                    index_doc
                                        .get_str("name")
                                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                                        .to_owned(),
                                );
                            } else {
                                already_sync.push(i);
                            }
                            continue;
                        }
                    }

                    if doc_are_eq(dbg!(&index_doc), dbg!(&existing_index)) {
                        already_sync.push(i);
                    } else {
                        // An index with the same specification already exists, we need to drop it.
                        to_drop.push(
                            index_doc
                                .get_str("name")
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                                .to_owned(),
                        );
                    }
                }
            }

            // Drop all remaining existing index expect "_id_" (for the "_id" key)
            // "_id" is special and cannot be deleted.
            // https://api.mongodb.com/wiki/current/Indexes.html#Indexes-The%5CidIndex
            for existing_index in existing_indexes.values() {
                let name = existing_index
                    .get_str("name")
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                    .to_owned();
                if name != "_id_" {
                    to_drop.push(name);
                }
            }

            if !to_drop.is_empty() {
                // Actually send the drop command
                // Dropping multiple indexes is available only starting MongoDB 4.2
                // If this fails, we fallback to a loop dropping all indexes individually
                // TODO: it would be better to select the method by checking mongo version, but db.version()
                // is not yet exposed by the driver.
                if h_run_command(
                    db,
                    doc! { "dropIndexes": CollConf::collection_name(), "index": &to_drop },
                )
                .await
                .is_err()
                {
                    for index_name in to_drop {
                        h_run_command(
                            db,
                            doc! { "dropIndexes": CollConf::collection_name(), "index": index_name },
                        )
                        .await?;
                    }
                }
            }

            // Ignore index already in sync
            for i in already_sync.into_iter().rev() {
                indexes.0.remove(i);
            }
        }
        Err(e) => {
            match e.kind.as_ref() {
                mongodb::error::ErrorKind::Command(err) if err.code == 26 => {
                    // Namespace doesn't exists yet as such no index is present either.
                }
                _ => return Err(e),
            }
        }
    }

    if !indexes.0.is_empty() {
        h_run_command(
            db,
            indexes.create_indexes_command(CollConf::collection_name()),
        )
        .await?;
    }

    Ok(())
}

async fn h_run_command(
    db: &Database,
    command_doc: Document,
) -> Result<Document, mongodb::error::Error> {
    let ret = db
        .run_command(
            command_doc,
            Some(SelectionCriteria::ReadPreference(ReadPreference::Primary)),
        )
        .await?;
    if let Ok(err) = from_bson::<mongodb::error::CommandError>(Bson::Document(ret.clone())) {
        Err(mongodb::error::Error::from(
            mongodb::error::ErrorKind::Command(err),
        ))
    } else {
        Ok(ret)
    }
}

#[derive(Deserialize)]
struct ListIndexesRet {
    pub cursor: Cursor,
}

#[derive(Deserialize)]
struct Cursor {
    pub id: i64,
    #[serde(rename = "firstBatch", default)]
    pub first_batch: Vec<Document>,
}

fn doc_are_eq(a: &Document, b: &Document) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for (key, a_val) in a {
        match b.get(key) {
            Some(b_val) if a_val != b_val => {
                return false;
            }
            Some(_) => {}
            None => {
                return false;
            }
        }
    }

    true
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
