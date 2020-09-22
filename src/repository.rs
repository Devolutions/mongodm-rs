//! Repositories are abstraction over a specific mongo collection for a given `Model`

use crate::{CollectionConfig, Model};
use mongodb::bson::from_bson;
use mongodb::bson::to_bson;
use mongodb::bson::Bson;
use mongodb::bson::Document;
use mongodb::options::*;
use mongodb::results::*;

/// Associate a `mongodb::Collection` and a specific `Model`.
///
/// This type can safely be copied and passed around because `std::sync::Arc` is used internally.
/// Underlying `mongodb::Collection` can be retrieved at anytime with `Repository::get_underlying`.
#[derive(Debug)]
pub struct Repository<M: Model> {
    coll: mongodb::Collection,
    _pd: std::marker::PhantomData<M>,
}

impl<M: Model> Clone for Repository<M> {
    fn clone(&self) -> Self {
        Self {
            coll: self.coll.clone(),
            _pd: std::marker::PhantomData,
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

        Self {
            coll,
            _pd: std::marker::PhantomData,
        }
    }

    /// Create a new repository with associated collection options (override `Model::coll_options`).
    pub fn new_with_options(db: mongodb::Database, options: CollectionOptions) -> Self {
        Self {
            coll: db.collection_with_options(M::CollConf::collection_name(), options),
            _pd: std::marker::PhantomData,
        }
    }

    /// Returns associated `M::collection_name`.
    pub fn collection_name(&self) -> &'static str {
        M::CollConf::collection_name()
    }

    /// Returns underlying `mongodb::Collection`.
    pub fn get_underlying(&self) -> mongodb::Collection {
        mongodb::Collection::clone(&self.coll)
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
            coll: self.coll,
            _pd: std::marker::PhantomData,
        }
    }

    /// Drops the underlying collection, deleting all data, users, and indexes stored inside.
    pub async fn drop(
        &self,
        options: impl Into<Option<DropCollectionOptions>>,
    ) -> mongodb::error::Result<()> {
        self.coll.drop(options).await
    }

    /// Runs an aggregation operation.
    ///
    /// [Mongo manual](https://docs.mongodb.com/manual/aggregation/)
    pub async fn aggregate(
        &self,
        pipeline: impl IntoIterator<Item = Document>,
        options: impl Into<Option<AggregateOptions>>,
    ) -> mongodb::error::Result<crate::cursor::ModelCursor<M>> {
        self.coll
            .aggregate(pipeline, options)
            .await
            .map(crate::cursor::ModelCursor::from)
    }

    /// Estimates the number of documents in the collection using collection metadata.
    pub async fn estimated_document_count(
        &self,
        options: impl Into<Option<EstimatedDocumentCountOptions>>,
    ) -> mongodb::error::Result<i64> {
        self.coll.estimated_document_count(options).await
    }

    /// Gets the number of documents matching `filter`.
    ///
    /// Note that using `Repository::estimated_document_count` is recommended instead of this method is most cases.
    pub async fn count_documents(
        &self,
        filter: impl Into<Option<Document>>,
        options: impl Into<Option<CountOptions>>,
    ) -> mongodb::error::Result<i64> {
        self.coll.count_documents(filter, options).await
    }

    /// Deletes all documents stored in the collection matching `query`.
    pub async fn delete_many(
        &self,
        query: Document,
        options: impl Into<Option<DeleteOptions>>,
    ) -> mongodb::error::Result<DeleteResult> {
        self.coll.delete_many(query, options).await
    }

    /// Deletes up to one document found matching `query`.
    pub async fn delete_one(
        &self,
        query: Document,
        options: impl Into<Option<DeleteOptions>>,
    ) -> mongodb::error::Result<DeleteResult> {
        self.coll.delete_one(query, options).await
    }

    /// Finds the distinct values of the field specified by `field_name` across the collection.
    pub async fn distinct(
        &self,
        field_name: &str,
        filter: impl Into<Option<Document>>,
        options: impl Into<Option<DistinctOptions>>,
    ) -> mongodb::error::Result<Vec<M>> {
        let bson_items = self.coll.distinct(field_name, filter, options).await?;
        let mut items = Vec::with_capacity(bson_items.len());
        for bson in bson_items {
            let item =
                from_bson(bson).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            items.push(item);
        }
        Ok(items)
    }

    /// Finds the documents in the collection matching `filter`.
    pub async fn find(
        &self,
        filter: impl Into<Option<Document>>,
        options: impl Into<Option<FindOptions>>,
    ) -> mongodb::error::Result<crate::cursor::ModelCursor<M>> {
        self.coll
            .find(filter, options)
            .await
            .map(crate::cursor::ModelCursor::from)
    }

    /// Finds a single document in the collection matching `filter`.
    pub async fn find_one(
        &self,
        filter: impl Into<Option<Document>>,
        options: impl Into<Option<FindOneOptions>>,
    ) -> mongodb::error::Result<Option<M>> {
        let doc_opt = self.coll.find_one(filter, options).await?;
        if let Some(doc) = doc_opt {
            let item = Self::h_doc_to_model(doc)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// Atomically finds up to one document in the collection matching `filter` and deletes it.
    pub async fn find_one_and_delete(
        &self,
        filter: Document,
        options: impl Into<Option<FindOneAndDeleteOptions>>,
    ) -> mongodb::error::Result<Option<M>> {
        let doc_opt = self.coll.find_one_and_delete(filter, options).await?;
        if let Some(doc) = doc_opt {
            let item = Self::h_doc_to_model(doc)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// Atomically finds up to one document in the collection matching `filter` and replaces it with
    /// `replacement`.
    pub async fn find_one_and_replace(
        &self,
        filter: Document,
        replacement: &M,
        options: impl Into<Option<FindOneAndReplaceOptions>>,
    ) -> mongodb::error::Result<Option<M>> {
        let replacement = Self::h_model_to_doc(replacement)?;
        let doc_opt = self
            .coll
            .find_one_and_replace(filter, replacement, options)
            .await?;
        if let Some(doc) = doc_opt {
            let item = Self::h_doc_to_model(doc)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// Atomically finds up to one model in the collection matching `filter` and updates it.
    ///
    /// Both `Document` and `Vec<Document>` implement `Into<UpdateModifications>`, so either can be
    /// passed in place of constructing the enum case. Note: pipeline updates are only supported
    /// in MongoDB 4.2+.
    pub async fn find_one_and_update(
        &self,
        filter: Document,
        update: impl Into<UpdateModifications>,
        options: impl Into<Option<FindOneAndUpdateOptions>>,
    ) -> mongodb::error::Result<Option<M>> {
        let doc_opt = self
            .coll
            .find_one_and_update(filter, update, options)
            .await?;
        if let Some(doc) = doc_opt {
            let item = Self::h_doc_to_model(doc)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// Inserts the models into the collection.
    pub async fn insert_many(
        &self,
        models: impl IntoIterator<Item = M>,
        options: impl Into<Option<InsertManyOptions>>,
    ) -> mongodb::error::Result<InsertManyResult> {
        let mut docs = Vec::new();
        for model in models.into_iter() {
            docs.push(Self::h_model_to_doc(&model)?)
        }

        self.coll.insert_many(docs, options).await
    }

    /// Inserts model `M` into the collection.
    pub async fn insert_one(
        &self,
        model: &M,
        options: impl Into<Option<InsertOneOptions>>,
    ) -> mongodb::error::Result<InsertOneResult> {
        let doc = Self::h_model_to_doc(model)?;
        self.coll.insert_one(doc, options).await
    }

    /// Replaces up to one document matching `query` in the collection with `replacement`.
    pub async fn replace_one(
        &self,
        query: Document,
        replacement: &M,
        options: impl Into<Option<ReplaceOptions>>,
    ) -> mongodb::error::Result<UpdateResult> {
        let replacement = Self::h_model_to_doc(replacement)?;
        self.coll.replace_one(query, replacement, options).await
    }

    /// Updates all documents matching `query` in the collection.
    ///
    /// Both `Document` and `Vec<Document>` implement `Into<UpdateModifications>`, so either can be
    /// passed in place of constructing the enum case. Note: pipeline updates are only supported
    /// in MongoDB 4.2+. See the official MongoDB
    /// [documentation](https://docs.mongodb.com/manual/reference/command/update/#behavior) for more information on specifying updates.
    pub async fn update_many(
        &self,
        query: Document,
        update: impl Into<UpdateModifications>,
        options: impl Into<Option<UpdateOptions>>,
    ) -> mongodb::error::Result<UpdateResult> {
        self.coll.update_many(query, update, options).await
    }

    /// Updates up to one document matching `query` in the collection.
    ///
    /// Both `Document` and `Vec<Document>` implement `Into<UpdateModifications>`, so either can be
    /// passed in place of constructing the enum case. Note: pipeline updates are only supported
    /// in MongoDB 4.2+. See the official MongoDB
    /// [documentation](https://docs.mongodb.com/manual/reference/command/update/#behavior) for more information on specifying updates.
    pub async fn update_one(
        &self,
        query: Document,
        update: impl Into<UpdateModifications>,
        options: impl Into<Option<UpdateOptions>>,
    ) -> mongodb::error::Result<UpdateResult> {
        self.coll.update_one(query, update, options).await
    }

    fn h_doc_to_model(doc: Document) -> mongodb::error::Result<M> {
        let item = from_bson(Bson::Document(doc))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(item)
    }

    fn h_model_to_doc(model: &M) -> mongodb::error::Result<Document> {
        let bson =
            to_bson(&model).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        if let Bson::Document(doc) = bson {
            Ok(doc)
        } else {
            Err(mongodb::error::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "model can't be serialized into a `Bson::Document`",
            )))
        }
    }
}
