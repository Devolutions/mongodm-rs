//! Repositories are abstraction over a specific mongo collection for a given `Model`

use crate::{CollectionConfig, Model};
use async_trait::async_trait;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, from_document, to_bson, Document};
use mongodb::error::Result;
use mongodb::options::*;
use serde::Deserialize;
use std::borrow::Borrow;
use std::ops::Deref;

/// Represents an individual update operation for the `bulk_update` function.
#[derive(Debug)]
pub struct BulkUpdate {
    pub query: Document,
    pub update: Document,
    pub options: Option<UpdateOptions>,
}

/// Result of a `bulk_update` operation.
#[derive(Debug, Deserialize)]
pub struct BulkUpdateResult {
    #[serde(rename = "n")]
    pub nb_affected: u64,
    #[serde(rename = "nModified")]
    pub nb_modified: u64,
    #[serde(default)]
    pub upserted: Vec<BulkUpdateUpsertResult>,
}

/// Individual update result of a `bulk_update` operation.
/// Contains the generated id in case of an upsert.
#[derive(Debug, Deserialize)]
pub struct BulkUpdateUpsertResult {
    pub index: u64,
    #[serde(alias = "_id")]
    pub id: ObjectId,
}

/// Associate a `mongodb::Collection` and a specific `Model`.
///
/// This type can safely be copied and passed around because `std::sync::Arc` is used internally.
/// Underlying `mongodb::Collection` can be retrieved at anytime with `Repository::get_underlying`.
#[derive(Debug)]
pub struct Repository<M: Model> {
    db: mongodb::Database, // FIXME: temporary keep reference to database object for `bulk_update` operation
    coll: mongodb::Collection<M>,
}

impl<M: Model> Deref for Repository<M> {
    type Target = mongodb::Collection<M>;
    fn deref(&self) -> &mongodb::Collection<M> {
        &self.coll
    }
}

impl<M: Model> Clone for Repository<M> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            coll: self.coll.clone_with_type(),
        }
    }
}

impl<M: Model> Repository<M> {
    /// Create a new repository from the given mongo client.
    pub fn new(db: mongodb::Database) -> Self {
        let coll = if let Some(options) = M::CollConf::collection_options() {
            db.collection_with_options(M::CollConf::collection_name(), options)
        } else {
            db.collection(M::CollConf::collection_name())
        };

        Self { db, coll }
    }

    /// Create a new repository with associated collection options (override `Model::coll_options`).
    pub fn new_with_options(db: mongodb::Database, options: CollectionOptions) -> Self {
        let coll = db.collection_with_options(M::CollConf::collection_name(), options);
        Self { db, coll }
    }

    /// Returns associated `M::collection_name`.
    pub fn collection_name(&self) -> &'static str {
        M::CollConf::collection_name()
    }

    /// Returns underlying `mongodb::Collection`.
    pub fn get_underlying(&self) -> mongodb::Collection<M> {
        self.coll.clone_with_type()
    }

    /// Convert this repository to use another `Model`. Only compiles if both `Model::CollConf` are identicals.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn demo() -> Result<(), mongodb::error::Error> {
    /// # use mongodm::mongo::{Client, options::ClientOptions};
    /// # let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    /// # let client = Client::with_options(client_options)?;
    /// # let db = client.database("mongodm_wayk_demo");
    /// use mongodm::{ToRepository, Model, CollectionConfig};
    /// use mongodm::mongo::bson::doc;
    /// use mongodm::f;
    /// use serde::{Serialize, Deserialize};
    ///
    /// struct UserCollConf;
    ///
    /// impl CollectionConfig for UserCollConf {
    ///     fn collection_name() -> &'static str {
    ///         "cast_model"
    ///     }
    /// }
    ///
    /// // Latest schema currently in use
    /// #[derive(Serialize, Deserialize)]
    /// struct User {
    ///     username: String,
    ///     last_seen: i64,
    /// }
    ///
    /// impl Model for User {
    ///     type CollConf = UserCollConf;
    /// }
    ///
    /// // Old schema
    /// #[derive(Serialize, Deserialize)]
    /// struct UserV1 {
    ///     name: String,
    ///     ls: i64,
    /// }
    ///
    /// // Versionned version of our `User`
    /// #[derive(Serialize, Deserialize)]
    /// #[serde(untagged)]
    /// enum UserVersionned {
    ///     Last(User),
    ///     V1(UserV1),
    /// }
    ///
    /// impl Model for UserVersionned {
    ///     type CollConf = UserCollConf; // same as the non-versionned version
    /// }
    ///
    /// // We have some repository for `User`
    /// let repo = db.repository::<User>();
    ///
    /// # let coll = repo.get_underlying();
    /// # coll.drop(None).await?;
    /// # coll.insert_one(doc!{ f!(name in UserV1): "Bernard", f!(ls in UserV1): 1500 }, None).await?;
    /// // Assume the following document is stored: { "name": "Bernard", "ls": 1500 }
    ///
    /// // Following query should fails because the schema doesn't match
    /// let err = repo.find_one(doc!{ f!(name in UserV1): "Bernard" }, None).await.err().unwrap();
    /// assert_eq!(err.to_string(), "missing field `username`"); // serde deserialization error
    ///
    /// // We can get a repository for `UserVersionned` from our `Repository<User>`
    /// // because `User::CollConf` == `UserVersionned::CollConf`
    /// let repo_versionned = repo.cast_model::<UserVersionned>();
    ///
    /// // Our versionned model should match with the document
    /// let ret = repo_versionned.find_one(doc!{ f!(name in UserV1): "Bernard" }, None).await?;
    /// match ret {
    ///     Some(UserVersionned::V1(UserV1 { name, ls: 1500 })) if name == "Bernard" => { /* success */ }
    ///     _ => panic!("Expected document was missing"),
    /// }
    ///
    /// # Ok(())
    /// # }
    /// # let mut rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(demo());
    /// ```
    ///
    /// Following code will fail to compile because `CollectionConfig` doesn't match.
    ///
    /// ```compile_fail
    /// # async fn demo() -> Result<(), mongodb::error::Error> {
    /// # use mongodm::mongo::{Client, options::ClientOptions};
    /// # let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    /// # let client = Client::with_options(client_options)?;
    /// # let db = client.database("mongodm_wayk_demo");
    /// use mongodm::{ToRepository, Model, CollectionConfig};
    /// use serde::{Serialize, Deserialize};
    ///
    /// struct ACollConf;
    ///
    /// impl CollectionConfig for ACollConf {
    ///     fn collection_name() -> &'static str { "a" }
    /// }
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct A;
    ///
    /// impl Model for A {
    ///     type CollConf = ACollConf;
    /// }
    ///
    /// struct BCollConf;
    ///
    /// impl CollectionConfig for BCollConf {
    ///     fn collection_name() -> &'static str { "B" }
    /// }
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct B;
    ///
    /// impl Model for B {
    ///     type CollConf = BCollConf;
    /// }
    ///
    /// // Doesn't compile because `A` and `B` doesn't share the same `CollectionConfig`.
    /// db.repository::<A>().cast_model::<B>();
    /// # Ok(())
    /// # }
    /// # let mut rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(demo());
    /// ```
    pub fn cast_model<OtherModel>(self) -> Repository<OtherModel>
    where
        OtherModel: Model<CollConf = M::CollConf>,
    {
        Repository {
            db: self.db,
            coll: self.coll.clone_with_type(),
        }
    }

    /// Apply multiple update operations in bulk.
    ///
    /// This will be removed once support for bulk update is added to the official driver.
    /// [see](https://jira.mongodb.org/browse/RUST-531) for tracking progress on this feature in the official driver.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use serde::{Serialize, Deserialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #     name: String,
    /// #     age: i64,
    /// # }
    /// # impl Model for User {
    /// #     type CollConf = UserCollConf;
    /// # }
    /// # struct UserCollConf;
    /// # impl CollectionConfig for UserCollConf {
    /// #     fn collection_name() -> &'static str { "user" }
    /// # }
    /// use mongodm::prelude::*;
    /// /* ... */
    /// # async fn demo(_db: mongodb::Database) {
    /// let db: mongodb::Database; /* exists */
    /// # db = _db;
    /// let repository = db.repository::<User>();
    /// /* ... */
    /// let bulk_update_res = repository
    ///     .bulk_update(&vec![
    ///         &BulkUpdate {
    ///             query: doc! { f!(name in User): "Dane" },
    ///             update: doc! { Set: { f!(age in User): 12 } },
    ///             options: None,
    ///         },
    ///         &BulkUpdate {
    ///             query: doc! { f!(name in User): "David" },
    ///             update: doc! { Set: { f!(age in User): 30 } },
    ///             options: None,
    ///         },
    ///     ])
    ///     .await
    ///     .unwrap();
    /// assert_eq!(bulk_update_res.nb_affected, 2);
    /// assert_eq!(bulk_update_res.nb_modified, 2);
    /// # }
    /// ```
    pub async fn bulk_update<V, U>(&self, updates: V) -> Result<BulkUpdateResult>
    where
        V: Borrow<Vec<U>> + Send + Sync,
        U: Borrow<BulkUpdate> + Send + Sync,
    {
        Ok(self.coll.bulk_update(&self.db, updates).await?)
    }
}

/// MongODM-provided utilities functions on `mongodb::Collection<M>`.
#[async_trait]
pub trait CollectionExt {
    /// Apply multiple update operations in bulk.
    ///
    /// This will be removed once support for bulk update is added to the official driver.
    /// [see](https://jira.mongodb.org/browse/RUST-531) for tracking progress on this feature in the official driver.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use serde::{Serialize, Deserialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User {
    /// #     name: String,
    /// #     age: i64,
    /// # }
    /// use mongodm::prelude::*;
    /// /* ... */
    /// # async fn demo(_db: mongodb::Database) {
    /// let db: mongodb::Database; /* exists */
    /// # db = _db;
    /// let collection = db.collection::<User>("user");
    /// /* ... */
    /// let bulk_update_res = collection
    ///     .bulk_update(&db, &vec![
    ///         &BulkUpdate {
    ///             query: doc! { f!(name in User): "Dane" },
    ///             update: doc! { Set: { f!(age in User): 12 } },
    ///             options: None,
    ///         },
    ///         &BulkUpdate {
    ///             query: doc! { f!(name in User): "David" },
    ///             update: doc! { Set: { f!(age in User): 30 } },
    ///             options: None,
    ///         },
    ///     ])
    ///     .await
    ///     .unwrap();
    /// assert_eq!(bulk_update_res.nb_affected, 2);
    /// assert_eq!(bulk_update_res.nb_modified, 2);
    /// # }
    /// ```
    async fn bulk_update<V, U>(
        &self,
        db: &mongodb::Database,
        updates: V,
    ) -> Result<BulkUpdateResult>
    where
        V: 'async_trait + Send + Sync + Borrow<Vec<U>>,
        U: 'async_trait + Send + Sync + Borrow<BulkUpdate>;
}

#[async_trait]
impl<M: Send + Sync> CollectionExt for mongodb::Collection<M> {
    async fn bulk_update<V, U>(
        &self,
        db: &mongodb::Database,
        updates: V,
    ) -> Result<BulkUpdateResult>
    where
        V: 'async_trait + Send + Sync + Borrow<Vec<U>>,
        U: 'async_trait + Send + Sync + Borrow<BulkUpdate>,
    {
        let updates = updates.borrow();
        let mut update_docs = Vec::with_capacity(updates.len());
        for u in updates {
            let u = u.borrow();
            let mut doc = doc! {
                "q": &u.query,
                "u": &u.update,
                "multi": false,
            };
            if let Some(options) = &u.options {
                if let Some(ref upsert) = options.upsert {
                    doc.insert("upsert", upsert);
                }
                if let Some(ref collation) = options.collation {
                    doc.insert("collation", to_bson(collation)?);
                }
                if let Some(ref array_filters) = options.array_filters {
                    doc.insert("arrayFilters", array_filters);
                }
                if let Some(ref hint) = options.hint {
                    doc.insert("hint", to_bson(hint)?);
                }
            }
            update_docs.push(doc);
        }
        let mut command = doc! {
            "update": self.name(),
            "updates": update_docs,
        };
        if let Some(ref write_concern) = self.write_concern() {
            command.insert("writeConcern", to_bson(write_concern)?);
        }
        let res = db.run_command(command).await?;
        Ok(from_document(res)?)
    }
}
