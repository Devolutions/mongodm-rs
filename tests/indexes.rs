#[macro_use]
extern crate pretty_assertions;

use mongodb::{bson::doc, options::ClientOptions, Client};
use mongodm::{f, Index, IndexOption, Indexes, Model, ToRepository};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ModelOne {
    field: String,
}

impl Model for ModelOne {
    fn collection_name() -> &'static str {
        "one_sync"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new(f!(field in ModelOne)).with_option(IndexOption::Unique))
    }
}

#[tokio::test]
#[ignore]
async fn one_sync() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("rust_mongo_orm_tests");

    let repository = db.repository::<ModelOne>();
    repository.drop(None).await.unwrap();
    repository.sync_indexes().await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": repository.collection_name() }, None)
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
    fn collection_name() -> &'static str {
        "multiple_sync"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new(f!(field in ModelMultiple))
                .with_key(f!(last_seen in ModelMultiple))
                .with_option(IndexOption::Unique),
        )
    }
}

#[derive(Serialize, Deserialize)]
struct ModelMultipleNoLastSeen {
    field: String,
}

impl Model for ModelMultipleNoLastSeen {
    fn collection_name() -> &'static str {
        "multiple_sync"
    }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new(f!(field in ModelMultipleNoLastSeen)).with_option(IndexOption::Unique))
    }
}

#[derive(Serialize, Deserialize)]
struct ModelMultipleNotUnique {
    field: String,
}

impl Model for ModelMultipleNotUnique {
    fn collection_name() -> &'static str {
        "multiple_sync"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new(f!(field in ModelMultipleNotUnique)))
    }
}

#[tokio::test]
#[ignore]
async fn multiple_sync() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("rust_mongo_orm_tests");

    let repository = db.repository::<ModelMultiple>();
    repository.drop(None).await.unwrap();
    repository.sync_indexes().await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": repository.collection_name() }, None)
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

    let repository = db.repository::<ModelMultipleNoLastSeen>();

    repository.sync_indexes().await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": repository.collection_name() }, None)
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

    let repository = db.repository::<ModelMultipleNotUnique>();

    repository.sync_indexes().await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": repository.collection_name() }, None)
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
