//! Indexes are used for efficient mongo queries.

use crate::CollectionConfig;

#[cfg(feature = "bson-3")]
use mongodb::bson::deserialize_from_bson;
#[cfg(feature = "compat-3-0-0")]
use mongodb::bson::from_bson as deserialize_from_bson;

use mongodb::Database;
use mongodb::bson::{Bson, Document, doc};
use mongodb::options::{ReadPreference, RunCommandOptions, SelectionCriteria};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;

/// Index sort order (useful for compound indexes).
///
/// [Mongo manual](https://docs.mongodb.com/manual/core/index-compound/#sort-order)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Self::SortIndex(s) => match s.direction {
                SortOrder::Ascending => format!("{}_1", s.name),
                SortOrder::Descending => format!("{}_-1", s.name),
            },

            Self::TextIndex(t) => format!("{}_text", t.name),
        }
    }

    fn get_name(&self) -> String {
        match self {
            Self::SortIndex(s) => s.name.to_string(),
            Self::TextIndex(t) => t.name.to_string(),
        }
    }

    fn get_value(&self) -> Bson {
        match self {
            Self::SortIndex(s) => s.direction.into(),
            Self::TextIndex(_) => "text".into(),
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
        self.add_key_with_direction(key, SortOrder::Ascending);
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
            Self::Background => "background",
            Self::Unique => "unique",
            Self::Name(..) => "name",
            Self::PartialFilterExpression(..) => "partialFilterExpression",
            Self::Sparse => "sparse",
            Self::ExpireAfterSeconds(..) => "expireAfterSeconds",
            Self::StorageEngine(..) => "storageEngine",
            Self::Collation(..) => "collation",
            Self::Weights(..) => "weights",
            Self::Custom { name, .. } => name.as_str(),
        }
    }

    pub fn into_value(self) -> Bson {
        match self {
            Self::Background | Self::Unique | Self::Sparse => Bson::Boolean(true),
            Self::Name(val) => Bson::String(val),
            Self::ExpireAfterSeconds(val) => Bson::Int32(val),
            Self::PartialFilterExpression(doc)
            | Self::StorageEngine(doc)
            | Self::Collation(doc) => Bson::Document(doc),
            Self::Weights(w) => {
                let mut doc = Document::new();
                for (k, v) in w {
                    doc.insert(k, Bson::from(v));
                }
                Bson::Document(doc)
            }
            Self::Custom { value, .. } => value,
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
            let parsed_ret: ListIndexesRet =
                deserialize_from_bson(Bson::Document(ret)).map_err(std::io::Error::other)?;

            if parsed_ret.cursor.id != 0 {
                // batch isn't complete
                return Err(std::io::Error::other(format!(
                    "couldn't list all indexes from '{}'",
                    CollConf::collection_name()
                ))
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
                let mut index_doc = if index
                    .keys
                    .iter()
                    .any(|ind| matches!(ind, IndexKey::TextIndex(_)))
                {
                    let mut doc = index.into_document();

                    // There can only be 1 text index per collection so when a text index is saved, the keys are automatically changed to this. We keep a copy for the weight comparison.
                    text_index_keys = doc.get("key").cloned();
                    doc.insert("key", doc! { "_fts": "text", "_ftsx": 1 });
                    doc
                } else {
                    index.into_document()
                };

                // Owned so the borrow on `index_doc` ends here: the collation rewrite below needs it
                // mutably. Only the comparison copy is touched — `indexes` still carries the
                // original declaration and is what `create_indexes_command` sends.
                let key = index_doc
                    .get("key")
                    .ok_or_else(|| std::io::Error::other("index doc is missing 'key'"))?
                    .to_string();
                if let Some(mut existing_index) = existing_indexes.remove(&key) {
                    // "ns" and "v" in the response should not be used for the comparison
                    existing_index.remove("ns");
                    existing_index.remove("v");

                    if let Some(declared_collation) =
                        index_doc.get_document("collation").ok().cloned()
                    {
                        let expanded = expanded_collation(
                            db,
                            CollConf::collection_name(),
                            &declared_collation,
                        )
                        .await?;
                        apply_expanded_collation(&mut index_doc, expanded);
                    }

                    // We compare the text index here, the keys become weights of 1 after saving in the DB. Custom weights not supported yet.
                    if let Some(Bson::Document(mut keys_to_set)) = text_index_keys
                        && let Some(Bson::Document(existing_weights)) =
                            existing_index.get("weights")
                    {
                        // Changing all text values to the default weight of 1
                        for keys in keys_to_set.iter_mut() {
                            match keys.1 {
                                Bson::String(t) if t == "text" => {
                                    *keys.1 = Bson::Int32(1);
                                }
                                _ => (),
                            }
                        }

                        if existing_weights.eq(&keys_to_set) {
                            already_sync.push(i);
                        } else {
                            to_drop.push(
                                index_doc
                                    .get_str("name")
                                    .map_err(std::io::Error::other)?
                                    .to_owned(),
                            );
                        }
                        continue;
                    }

                    if doc_are_eq(&index_doc, &existing_index) {
                        already_sync.push(i);
                    } else {
                        // An index with the same specification already exists, we need to drop it.
                        to_drop.push(
                            index_doc
                                .get_str("name")
                                .map_err(std::io::Error::other)?
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
                    .map_err(std::io::Error::other)?
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
    let primary_options = RunCommandOptions::builder()
        .selection_criteria(SelectionCriteria::ReadPreference(ReadPreference::Primary))
        .build();

    let ret = db
        .run_command(command_doc)
        .with_options(primary_options)
        .await?;
    deserialize_from_bson::<mongodb::error::CommandError>(Bson::Document(ret.clone())).map_or_else(
        |_| Ok(ret),
        |err| {
            Err(mongodb::error::Error::from(
                mongodb::error::ErrorKind::Command(err),
            ))
        },
    )
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

/// Asks the server how it would expand a declared collation.
///
/// `MongoDB` fills in every ICU field plus a `version` of its own when it stores a collation, and
/// which values it picks depends on the locale: `fr_CA` defaults `backwards` to true, `da` defaults
/// `caseFirst` to `upper`, `th` and `vi` default `normalization` to true. A local table of defaults
/// is wrong for every locale it does not list, so the server is asked instead — `explain` reports
/// the very expansion it would store.
///
/// `None` means the server reports no collation, which is how it answers `locale: "simple"`: that
/// request is stored as an index carrying no collation at all.
async fn expanded_collation(
    db: &Database,
    collection: &str,
    collation: &Document,
) -> Result<Option<Document>, mongodb::error::Error> {
    let ret = h_run_command(
        db,
        doc! {
            "explain": { "find": collection, "filter": {}, "collation": collation },
            "verbosity": "queryPlanner",
        },
    )
    .await?;

    Ok(ret
        .get_document("queryPlanner")
        .ok()
        .and_then(|planner| planner.get_document("collation").ok().cloned()))
}

/// Replaces the declared collation with the server's expansion of it so the two can be compared.
///
/// Both sides then hold a document the server itself produced, which is the only way the comparison
/// can work: a declaration of `{"locale": "en", "strength": 2}` can never equal the ten fields
/// `listIndexes` reports, so without this `sync_indexes` drops and recreates the index on every run.
/// Any real change to the declaration expands differently and is still caught.
///
/// `None` drops the key, matching an index the server chose to store without a collation. Taking the
/// expansion as an argument rather than fetching it keeps this half testable without a server.
fn apply_expanded_collation(declared: &mut Document, expanded: Option<Document>) {
    match expanded {
        Some(collation) => {
            declared.insert("collation", collation);
        }
        None => {
            declared.remove("collation");
        }
    }
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

    /// Expansions captured from `MongoDB` 7 with ICU 57.1. `en` is the plain case; the rest are
    /// locales whose defaults differ from it, which is why the expansion has to come from the server
    /// rather than a table: `fr_CA` flips `backwards`, `da` sets `caseFirst` to `upper`, `th` sets
    /// both `alternate` to `shifted` and `normalization` to true.
    fn server_expansion(locale: &str) -> Document {
        let (case_first, alternate, normalization, backwards) = match locale {
            "fr_CA" => ("off", "non-ignorable", false, true),
            "da" => ("upper", "non-ignorable", false, false),
            "th" => ("off", "shifted", true, false),
            _ => ("off", "non-ignorable", false, false),
        };

        doc! {
            "locale": locale,
            "caseLevel": false,
            "caseFirst": case_first,
            "strength": 3,
            "numericOrdering": false,
            "alternate": alternate,
            "maxVariable": "punct",
            "normalization": normalization,
            "backwards": backwards,
            "version": "57.1",
        }
    }

    fn stored_index(collation: Option<Document>) -> Document {
        let mut index = doc! { "key": { "field": 1 }, "name": "collated_field" };
        if let Some(collation) = collation {
            index.insert("collation", collation);
        }

        index
    }

    fn declared_index(collation: Document) -> Document {
        Index::new("field")
            .with_option(IndexOption::Name("collated_field".to_owned()))
            .with_option(IndexOption::Collation(collation))
            .into_document()
    }

    /// The whole point: a declaration the server expanded has to compare equal to what it stored, or
    /// `sync_indexes` drops and recreates the index on every run.
    #[test]
    fn an_expanded_declaration_matches_the_stored_index() {
        let mut declared = declared_index(doc! { "locale": "en", "strength": 2 });

        apply_expanded_collation(&mut declared, Some(server_expansion("en")));

        assert!(
            doc_are_eq(&declared, &stored_index(Some(server_expansion("en")))),
            "expanded declaration {declared:?} should match the stored index"
        );
    }

    /// The locales a hard-coded default table gets wrong. Nothing here is locale-aware any more, so
    /// these pass for the same reason `en` does — worth pinning so nobody reintroduces a table.
    #[test]
    fn locales_with_unusual_defaults_match_their_stored_index() {
        for locale in ["fr_CA", "da", "th"] {
            let mut declared = declared_index(doc! { "locale": locale });

            apply_expanded_collation(&mut declared, Some(server_expansion(locale)));

            assert!(
                doc_are_eq(&declared, &stored_index(Some(server_expansion(locale)))),
                "locale `{locale}` should match its stored index, got {declared:?}"
            );
        }
    }

    /// `locale: "simple"` is stored as an index with no collation, and the server reports no
    /// expansion for it, so the declaration has to lose the key too.
    #[test]
    fn no_expansion_drops_the_declared_collation() {
        let mut declared = declared_index(doc! { "locale": "simple" });

        apply_expanded_collation(&mut declared, None);

        assert!(
            !declared.contains_key("collation"),
            "the collation should have been dropped, got {declared:?}"
        );
        assert!(
            doc_are_eq(&declared, &stored_index(None)),
            "a simple-locale declaration should match a collation-less index, got {declared:?}"
        );
    }

    /// A declaration that really did change expands differently and still has to rebuild — including
    /// the case that a plain declared-keys filter got wrong, where a field is dropped from the
    /// declaration and the stored index keeps a non-default value for it.
    #[test]
    fn a_changed_declaration_is_still_detected() {
        let cases = [
            ("strength", doc! { "locale": "en", "strength": 2 }),
            ("locale", doc! { "locale": "fr_CA" }),
            ("dropped strength", doc! { "locale": "en" }),
        ];

        for (what, declaration) in cases {
            let mut declared = declared_index(declaration);
            let stored = stored_index(Some({
                let mut collation = server_expansion("en");
                collation.insert("strength", 2);
                collation
            }));

            // The server expands whatever was declared; for these it is not what is stored.
            let expanded = match declared
                .get_document("collation")
                .unwrap()
                .get_str("locale")
            {
                Ok("fr_CA") => server_expansion("fr_CA"),
                _ => {
                    let mut collation = server_expansion("en");
                    if declared
                        .get_document("collation")
                        .unwrap()
                        .contains_key("strength")
                    {
                        collation.insert("strength", 2);
                    }
                    collation
                }
            };

            apply_expanded_collation(&mut declared, Some(expanded));

            let converged = doc_are_eq(&declared, &stored);
            assert_eq!(
                converged,
                what == "strength",
                "`{what}`: expected converged={}, got {converged} for {declared:?}",
                what == "strength"
            );
        }
    }
}
