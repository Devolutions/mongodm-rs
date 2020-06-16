//! Repositories are abstraction over a specific mongo collection for a given `Model`

use crate::DatabaseConfig;
use crate::DatabaseConfigExt;
use crate::Model;
use mongodb::bson::de::from_bson;
use mongodb::bson::doc;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use mongodb::options::CollectionOptions;
use mongodb::options::ReadPreference;
use mongodb::options::SelectionCriteria;
use serde::Deserialize;
use std::collections::HashMap;

/// Associate a mongo client to a `Database` and a `Model`.
///
/// This type can safely be copied and passed around. This is only wrapping a `mongodb::Collection` (internally using an `Arc`).
#[derive(Debug, Clone)]
pub struct Repository<B: DatabaseConfig, M: Model> {
    db: mongodb::Database, // TODO: once indexes are officially supported in the driver, we should get all required opterations in `Collection` and remove this field
    coll: mongodb::Collection,
    _pd: std::marker::PhantomData<(B, M)>,
}

impl<B: DatabaseConfig + DatabaseConfigExt, M: Model> Repository<B, M> {
    /// Create a new repository from the given mongo client.
    pub fn new(client: mongodb::Client) -> Self {
        let db = B::get_database(&client);

        let coll = if let Some(options) = M::coll_options() {
            db.collection_with_options(M::coll_name(), options)
        } else {
            db.collection(M::coll_name())
        };

        Self {
            db,
            coll,
            _pd: std::marker::PhantomData,
        }
    }

    /// Create a new repository with associated collection options (override `Model::coll_options`)
    pub fn new_with_options(client: mongodb::Client, options: CollectionOptions) -> Self {
        let db = B::get_database(&client);
        Self {
            coll: db.collection_with_options(M::coll_name(), options),
            db,
            _pd: std::marker::PhantomData,
        }
    }

    /// Returns associated `B::DB_NAME`
    pub fn db_name(&self) -> &'static str {
        B::db_name()
    }

    /// Returns associated `M::COLL_NAME`
    pub fn coll_name(&self) -> &'static str {
        M::coll_name()
    }

    /// Returns underlying `mongodb::Collection`
    pub fn get_underlying(&self) -> mongodb::Collection {
        mongodb::Collection::clone(&self.coll)
    }

    /// Synchronize model with underlying mongo collection.
    ///
    /// This should be called once per model on startup to synchronize indexes defined
    /// by the `Model`. Indexes found in the backend and not defined in the model are
    /// destroyed except for the special index "_id".
    pub async fn sync_indexes(&self) -> Result<(), mongodb::error::Error> {
        let mut indexes = M::indexes();

        match self
            .h_run_command(doc! { "listIndexes": M::coll_name() })
            .await
        {
            Ok(ret) => {
                let parsed_ret: ListIndexesRet = from_bson(Bson::Document(ret))
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                if parsed_ret.cursor.id != 0 {
                    // batch isn't complete
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("couldn't list all indexes from '{}'", M::coll_name()),
                    ))?;
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
                    let index_doc = index.into_document();
                    let key = index_doc.get("key").ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::Other, "index doc is missing 'key'")
                    })?;
                    if let Some(mut existing_index) = existing_indexes.remove(&key.to_string()) {
                        // "ns" and "v" in the response should not be used for the comparison
                        existing_index.remove("ns");
                        existing_index.remove("v");

                        if doc_are_eq(&index_doc, &existing_index) {
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
                    if self
                        .h_run_command(doc! { "dropIndexes": M::coll_name(), "index": &to_drop })
                        .await
                        .is_err()
                    {
                        for index_name in to_drop {
                            self.h_run_command(
                                doc! { "dropIndexes": M::coll_name(), "index": index_name },
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
                    mongodb::error::ErrorKind::CommandError(err) if err.code == 26 => {
                        // Namespace doesn't exists yet as such no index is present either.
                    }
                    _ => return Err(e),
                }
            }
        }

        if !indexes.0.is_empty() {
            self.h_run_command(indexes.create_indexes_command(M::coll_name()))
                .await?;
        }

        Ok(())
    }

    async fn h_run_command(
        &self,
        command_doc: Document,
    ) -> Result<Document, mongodb::error::Error> {
        let ret = self
            .db
            .run_command(
                command_doc,
                Some(SelectionCriteria::ReadPreference(ReadPreference::Primary)),
            )
            .await?;
        if let Ok(err) = from_bson::<mongodb::error::CommandError>(Bson::Document(ret.clone())) {
            Err(mongodb::error::Error::from(
                mongodb::error::ErrorKind::CommandError(err),
            ))
        } else {
            Ok(ret)
        }
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
