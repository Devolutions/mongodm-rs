#[macro_use]
extern crate pretty_assertions;

use mongodb::{bson::doc, options::ClientOptions, Client};
use mongodm::{Database, DatabaseExt, Index, IndexOption, Indexes, Model};
use serde::{Deserialize, Serialize};

struct TestDb;

impl Database for TestDb {
    const DB_NAME: &'static str = "rust_mongo_orm_tests";
}

#[derive(Serialize, Deserialize)]
struct ModelOne {
    field: String,
}

impl Model for ModelOne {
    const COLL_NAME: &'static str = "one_sync";

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new("field").with_option(IndexOption::Unique))
    }
}

#[tokio::test]
#[ignore]
async fn one_sync() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();

    let repository = TestDb::get_repository::<ModelOne>(client.clone());

    let db = client.database(repository.db_name());
    db.collection(repository.coll_name())
        .drop(None)
        .await
        .unwrap();

    repository.sync_indexes().await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": repository.coll_name() }, None)
        .await
        .unwrap();

    assert_eq!(
        ret,
        doc! {
            "cursor" : {
                "id" : 0u64,
                "ns" : "rust_mongo_orm_tests.one_sync",
                "firstBatch" : [
                    {
                        "v" : 2,
                        "key" : {
                            "_id" : 1
                        },
                        "name" : "_id_",
                        "ns" : "rust_mongo_orm_tests.one_sync"
                    },
                    {
                        "v" : 2,
                        "unique" : true,
                        "key" : {
                            "field" : 1
                        },
                        "name" : "field_1",
                        "ns" : "rust_mongo_orm_tests.one_sync"
                    }
                ]
            },
            "ok" : 1.0
        }
    );
}

#[derive(Serialize, Deserialize)]
struct ModelMultiple {
    field: String,
    last_seen: i64,
}

impl Model for ModelMultiple {
    const COLL_NAME: &'static str = "multiple_sync";

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new("field")
                .with_key("last_seen")
                .with_option(IndexOption::Unique),
        )
    }
}

#[derive(Serialize, Deserialize)]
struct ModelMultipleNoLastSeen {
    field: String,
}

impl Model for ModelMultipleNoLastSeen {
    const COLL_NAME: &'static str = "multiple_sync";

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new("field").with_option(IndexOption::Unique))
    }
}

#[derive(Serialize, Deserialize)]
struct ModelMultipleNotUnique {
    field: String,
}

impl Model for ModelMultipleNotUnique {
    const COLL_NAME: &'static str = "multiple_sync";

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new("field"))
    }
}

#[tokio::test]
#[ignore]
async fn multiple_sync() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();

    let repository = TestDb::get_repository::<ModelMultiple>(client.clone());

    let db = client.database(repository.db_name());
    db.collection(repository.coll_name())
        .drop(None)
        .await
        .unwrap();

    repository.sync_indexes().await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": repository.coll_name() }, None)
        .await
        .unwrap();

    assert_eq!(
        ret,
        doc! {
            "cursor" : {
                "id" : 0u64,
                "ns" : "rust_mongo_orm_tests.multiple_sync",
                "firstBatch" : [
                    {
                        "v" : 2,
                        "key" : {
                            "_id" : 1
                        },
                        "name" : "_id_",
                        "ns" : "rust_mongo_orm_tests.multiple_sync"
                    },
                    {
                        "v" : 2,
                        "unique" : true,
                        "key" : {
                            "field" : 1,
                            "last_seen" : 1
                        },
                        "name" : "field_1_last_seen_1",
                        "ns" : "rust_mongo_orm_tests.multiple_sync"
                    }
                ]
            },
            "ok" : 1.0
        }
    );

    let repository = TestDb::get_repository::<ModelMultipleNoLastSeen>(client.clone());

    repository.sync_indexes().await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": repository.coll_name() }, None)
        .await
        .unwrap();

    assert_eq!(
        ret,
        doc! {
            "cursor" : {
                "id" : 0u64,
                "ns" : "rust_mongo_orm_tests.multiple_sync",
                "firstBatch" : [
                    {
                        "v" : 2,
                        "key" : {
                            "_id" : 1
                        },
                        "name" : "_id_",
                        "ns" : "rust_mongo_orm_tests.multiple_sync"
                    },
                    {
                        "v" : 2,
                        "unique" : true,
                        "key" : {
                            "field" : 1,
                        },
                        "name" : "field_1",
                        "ns" : "rust_mongo_orm_tests.multiple_sync"
                    }
                ]
            },
            "ok" : 1.0
        }
    );

    let repository = TestDb::get_repository::<ModelMultipleNotUnique>(client.clone());

    repository.sync_indexes().await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": repository.coll_name() }, None)
        .await
        .unwrap();

    assert_eq!(
        ret,
        doc! {
            "cursor" : {
                "id" : 0u64,
                "ns" : "rust_mongo_orm_tests.multiple_sync",
                "firstBatch" : [
                    {
                        "v" : 2,
                        "key" : {
                            "_id" : 1
                        },
                        "name" : "_id_",
                        "ns" : "rust_mongo_orm_tests.multiple_sync"
                    },
                    {
                        "v" : 2,
                        "key" : {
                            "field" : 1,
                        },
                        "name" : "field_1",
                        "ns" : "rust_mongo_orm_tests.multiple_sync"
                    }
                ]
            },
            "ok" : 1.0
        }
    );
}
