use pretty_assertions::assert_eq;

use mongodb::bson::{Document, doc};
use mongodb::options::ClientOptions;
use mongodb::{Client, Database};
use mongodm::{CollectionConfig, Index, IndexOption, Indexes, sync_indexes};

async fn list_indexes(db: &Database, collection_name: &str) -> Vec<Document> {
    db.run_command(doc! { "listIndexes": collection_name })
        .await
        .unwrap()
        .get_document("cursor")
        .unwrap()
        .get_array("firstBatch")
        .unwrap()
        .iter()
        .map(|index| index.as_document().unwrap().clone())
        .collect()
}

fn assert_synced_indexes(indexes: &[Document], expected_key: Document, expected_unique: bool) {
    assert_eq!(indexes.len(), 2);

    let id_key = doc! { "_id": 1 };
    assert!(
        indexes
            .iter()
            .any(|index| index.get_document("key").is_ok_and(|key| key == &id_key))
    );

    let expected_index = indexes
        .iter()
        .find(|index| {
            index
                .get_document("key")
                .is_ok_and(|key| key == &expected_key)
        })
        .unwrap();

    assert_eq!(
        expected_index.get_bool("unique").ok(),
        expected_unique.then_some(true)
    );
}

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

    let indexes = list_indexes(&db, OneSyncCollConf::collection_name()).await;
    assert_synced_indexes(&indexes, doc! { "field": 1 }, true);
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

    let indexes = list_indexes(&db, MultipleSyncCollConf::collection_name()).await;
    assert_synced_indexes(&indexes, doc! { "field": 1, "last_seen": 1 }, true);

    sync_indexes::<MultipleNoLastSeenCollConf>(&db)
        .await
        .unwrap();

    let indexes = list_indexes(&db, MultipleNoLastSeenCollConf::collection_name()).await;
    assert_synced_indexes(&indexes, doc! { "field": 1 }, true);

    sync_indexes::<MultipleNotUniqueCollConf>(&db)
        .await
        .unwrap();

    let indexes = list_indexes(&db, MultipleNotUniqueCollConf::collection_name()).await;
    assert_synced_indexes(&indexes, doc! { "field": 1 }, false);
}
