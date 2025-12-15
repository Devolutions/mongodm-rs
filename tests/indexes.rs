use pretty_assertions::assert_eq;

use mongodb::Client;
use mongodb::bson::{Document, doc};
use mongodb::options::ClientOptions;
use mongodm::{CollectionConfig, Index, IndexOption, Indexes, sync_indexes};

struct OneSyncCollConf;

impl CollectionConfig for OneSyncCollConf {
    fn collection_name() -> &'static str {
        "one_sync"
    }

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
    let db = client.database("rust_mongo_orm_tests");

    db.collection::<Document>(OneSyncCollConf::collection_name())
        .drop()
        .await
        .unwrap();

    sync_indexes::<OneSyncCollConf>(&db).await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": OneSyncCollConf::collection_name() })
        .await
        .unwrap();

    assert_eq!(
        ret,
        doc! {
            "cursor" : {
                "id" : 0i64,
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

struct MultipleSyncCollConf;

impl CollectionConfig for MultipleSyncCollConf {
    fn collection_name() -> &'static str {
        "multiple_sync"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new("field")
                .with_key("last_seen")
                .with_option(IndexOption::Unique),
        )
    }
}

struct MultipleNoLastSeenCollConf;

impl CollectionConfig for MultipleNoLastSeenCollConf {
    fn collection_name() -> &'static str {
        "multiple_sync"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(Index::new("field").with_option(IndexOption::Unique))
    }
}

struct MultipleNotUniqueCollConf;

impl CollectionConfig for MultipleNotUniqueCollConf {
    fn collection_name() -> &'static str {
        "multiple_sync"
    }

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
    let db = client.database("rust_mongo_orm_tests");

    db.collection::<Document>(MultipleSyncCollConf::collection_name())
        .drop()
        .await
        .unwrap();

    sync_indexes::<MultipleSyncCollConf>(&db).await.unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": MultipleSyncCollConf::collection_name() })
        .await
        .unwrap();

    assert_eq!(
        ret,
        doc! {
            "cursor" : {
                "id" : 0i64,
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

    sync_indexes::<MultipleNoLastSeenCollConf>(&db)
        .await
        .unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": MultipleNoLastSeenCollConf::collection_name() })
        .await
        .unwrap();

    assert_eq!(
        ret,
        doc! {
            "cursor" : {
                "id" : 0i64,
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

    sync_indexes::<MultipleNotUniqueCollConf>(&db)
        .await
        .unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": MultipleNotUniqueCollConf::collection_name() })
        .await
        .unwrap();

    assert_eq!(
        ret,
        doc! {
            "cursor" : {
                "id" : 0i64,
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
