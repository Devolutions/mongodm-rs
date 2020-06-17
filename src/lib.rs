//! MongODM
//! =======
//!
//! A thin ODM layer for MongoDB built upon the [official Rust driver](https://github.com/mongodb/mongo-rust-driver).
//!
//! Main features:
//!
//! - A stronger API leveraging Rust type system
//! - Data structure models are defined using the well-known [`serde`](https://github.com/serde-rs/serde) serialization framework
//! - Index support on top of the `Database::run_command` (index management is currently not implemented in the underlying driver)
//! - Indexes synchronization
//! - Additional compile-time checks for queries using macros and type associated to mongo operators (eg: `And` instead of "$and")
//!
//! ## Example
//!
//! ```ignore
//! # async fn demo() -> Result<(), mongodb::error::Error> {
//! use mongodm::{DatabaseConfig, DatabaseConfigExt, Model, Indexes, Index, IndexOption};
//! use mongodm::mongo::{Client, options::ClientOptions, bson::doc};
//! use serde::{Serialize, Deserialize};
//! use std::borrow::Cow;
//!
//! struct WaykDb;
//!
//! impl DatabaseConfig for WaykDb {
//!     fn db_name(&self) -> Cow<'static, str> {
//!         Cow::Borrowed("mongodm_wayk_demo")
//!     }
//! }
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct User {
//!     username: String,
//!     last_seen: i64,
//! }
//!
//! impl Model for User {
//!     fn coll_name() -> &'static str {
//!         "user"
//!     }
//!
//!     fn indexes() -> Indexes {
//!         Indexes::new().with(Index::new("username").with_option(IndexOption::Unique))
//!     }
//! }
//!
//! let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
//! let client = Client::with_options(client_options)?;
//!
//! let repository = WaykDb.repository::<User>(client);
//! repository.sync_indexes().await?;
//! // indexes are now synced in backend
//!
//! let user = User {
//!     username: String::from("David"),
//!     last_seen: 1000,
//! };
//! repository.insert_one(&user, None).await?;
//!
//! // field! is used to make sure at compile time that `username` is a field of `User`
//! use mongodm::field;
//! let fetched_user = repository.find_one(doc! { field!(username in User): "David" }, None).await?;
//! assert!(fetched_user.is_some());
//! assert_eq!(fetched_user.unwrap(), user);
//!
//! // f! is a shorter version of field!
//! use mongodm::f;
//! repository.find_one(doc! { f!(username in User): "David" }, None).await?.unwrap();
//!
//! // With static operators for queries (prevent invalid queries due to typos)
//! use mongodm::operator::*;
//! repository.find_one(
//!     doc! { And: [
//!         { f!(username in User): "David" },
//!         { f!(last_seen in User): { GreaterThan: 500 } },
//!     ] },
//!     None
//! ).await?.unwrap();
//! # Ok(())
//! # }
//! # let mut rt = tokio::runtime::Runtime::new().unwrap();
//! # rt.block_on(demo());
//! ```

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;

mod macros;

pub mod cursor;
pub mod index;
pub mod operator;
pub mod repository;

pub use cursor::ModelCursor;
pub use index::{Index, IndexOption, Indexes};
pub use repository::Repository;

// Re-export mongodb
pub use mongodb as mongo;

/// Define collection configuration and associated indexes
pub trait Model: serde::ser::Serialize + serde::de::DeserializeOwned {
    /// Collection name to use when creating a `mongodb::Collection` instance
    fn coll_name() -> &'static str;

    /// `mongodb::options::CollectionOptions` to be used when creating a `mongodb::Collection` instance
    ///
    /// This method has a default implementation returning `None`. In such case configuration is defined by `DatabaseConfig::db_options`.
    fn coll_options() -> Option<mongodb::options::CollectionOptions> {
        None
    }

    /// Configure how indexes should be created and synchronized for the associated collection
    fn indexes() -> index::Indexes {
        index::Indexes::default()
    }
}

/// Define database configuration. Can be made into a trait object.
pub trait DatabaseConfig {
    /// Database name to use when creating a `mongodb::Database` instance.
    fn db_name(&self) -> std::borrow::Cow<'static, str>;

    /// `mongodb::options::DatabaseConfig` to be used when creating a `mongodb::Database` instance.
    ///
    /// This method has a default implementation returning `None`. In this case, `mongodb::Client` configuration will be applied.
    fn db_options(&self) -> Option<mongodb::options::DatabaseOptions> {
        None
    }
}

/// Add helper methods to `DatabaseConfig`. Auto-implemented for any type implementing `DatabaseConfig` trait.
///
/// Note: this is provided as a trait extension because it's not always desirable to get these in your namespace if
/// you prefer to implement your own helpers over your `DatabaseConfig` type.
pub trait DatabaseConfigExt: DatabaseConfig + Sized {
    /// Get a `mongodb::Database` configured as specified by `DatabaseConfig` trait.
    fn database(&self, client: &mongodb::Client) -> mongodb::Database {
        if let Some(options) = self.db_options() {
            client.database_with_options(self.db_name().as_ref(), options)
        } else {
            client.database(self.db_name().as_ref())
        }
    }

    /// Shorthand for `Repository::<Db, Model>::new`.
    fn repository<M: Model>(&self, client: mongodb::Client) -> Repository<Self, M> {
        Repository::new(client, self)
    }

    /// Shorthand for `Repository::<Db, Model>::new_with_options`.
    fn repository_with_options<M: Model>(
        &self,
        client: mongodb::Client,
        options: mongodb::options::CollectionOptions,
    ) -> Repository<Self, M> {
        Repository::new_with_options(client, self, options)
    }
}

impl<T> DatabaseConfigExt for T where T: DatabaseConfig {}
