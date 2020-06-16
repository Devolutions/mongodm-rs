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
//!
//! ## Example
//!
//! ```ignore
//! # async fn demo() -> Result<(), mongodb::error::Error> {
//! use mongodm::{Database, DatabaseExt, Model, Indexes, Index, IndexOption};
//! use mongodb::{Client, options::ClientOptions};
//! use serde::{Serialize, Deserialize};
//!
//! struct WaykDb;
//!
//! impl Database for WaykDb {
//!     const DB_NAME: &'static str = "mongodm_wayk_demo";
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     username: String,
//!     last_seen: i64,
//! }
//!
//! impl Model for User {
//!     const COLL_NAME: &'static str = "user";
//!
//!     fn indexes() -> Indexes {
//!         Indexes::new().with(Index::new("username").with_option(IndexOption::Unique))
//!     }
//! }
//!
//! let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
//! let client = Client::with_options(client_options)?;
//!
//! let repository = WaykDb::get_repository::<User>(client);
//! repository.sync_indexes().await?;
//! // indexes are now synced in backend
//! # Ok(())
//! # }
//! # let mut rt = tokio::runtime::Runtime::new().unwrap();
//! # rt.block_on(demo());
//! ```

#[macro_use]
#[cfg(test)]
extern crate pretty_assertions;

pub mod index;
pub mod repository;

pub use index::*;
pub use repository::*;

/// Define collection name and associated indexes
pub trait Model: serde::ser::Serialize + serde::de::DeserializeOwned {
    const COLL_NAME: &'static str;

    /// Configure how indexes should be synchronized for the associated collection
    fn indexes() -> index::Indexes {
        index::Indexes::default()
    }
}

/// Statically define database name
pub trait Database: Sized {
    const DB_NAME: &'static str;
}

/// Add helper methods to `Database`. Auto-implemented for any type implementing `Database` trait
pub trait DatabaseExt: Database {
    /// Shorthand for `Repository::<Db, Model>::new`
    fn get_repository<M: Model>(client: mongodb::Client) -> Repository<Self, M> {
        Repository::new(client)
    }

    /// Shorthand for `Repository::<Db, Model>::new_with_options`
    fn get_repository_with_options<M: Model>(
        client: mongodb::Client,
        options: mongodb::options::CollectionOptions,
    ) -> Repository<Self, M> {
        Repository::new_with_options(client, options)
    }
}

impl<T> DatabaseExt for T where T: Database {}
