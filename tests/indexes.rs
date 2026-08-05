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

fn collated_index() -> Indexes {
    Indexes::new().with(
        Index::new("field")
            .with_option(IndexOption::Name("collated_field".to_owned()))
            .with_option(IndexOption::Collation(
                doc! { "locale": "en", "strength": 2 },
            )),
    )
}

// A collection per test: libtest runs these concurrently under `--ignored`, and each one drops its
// collection on the way in.
struct CollatedSyncCollConf;

impl CollectionConfig for CollatedSyncCollConf {
    fn collection_name() -> &'static str {
        "collated_sync"
    }

    fn indexes() -> Indexes {
        collated_index()
    }
}

struct CollatedExpansionCollConf;

impl CollectionConfig for CollatedExpansionCollConf {
    fn collection_name() -> &'static str {
        "collated_expansion"
    }

    fn indexes() -> Indexes {
        collated_index()
    }
}

/// `accesses.since` is the point from which MongoDB gathered statistics for an index, so it is
/// reset by a recreate and left alone by a genuine no-op.
async fn index_stats_since(
    db: &mongodb::Database,
    collection: &str,
    index_name: &str,
) -> mongodb::bson::Bson {
    let mut cursor = db
        .collection::<Document>(collection)
        .aggregate(vec![doc! { "$indexStats": {} }])
        .await
        .unwrap();

    while cursor.advance().await.unwrap() {
        let stats = cursor.deserialize_current().unwrap();
        if stats.get_str("name") == Ok(index_name) {
            return stats
                .get_document("accesses")
                .unwrap()
                .get("since")
                .expect("$indexStats should report accesses.since")
                .clone();
        }
    }

    panic!("index `{index_name}` should exist after sync_indexes");
}

/// Documents the server behaviour the sync test depends on, and kept separate from it so a failure
/// tells you which of the two broke: the server no longer expanding collations, or `sync_indexes`
/// mishandling the expansion.
#[tokio::test]
#[ignore]
async fn listindexes_returns_more_collation_fields_than_were_declared() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("rust_mongo_orm_tests");

    db.collection::<Document>(CollatedExpansionCollConf::collection_name())
        .drop()
        .await
        .unwrap();

    sync_indexes::<CollatedExpansionCollConf>(&db)
        .await
        .unwrap();

    let ret = db
        .run_command(doc! { "listIndexes": CollatedExpansionCollConf::collection_name() })
        .await
        .unwrap();

    let stored = ret
        .get_document("cursor")
        .unwrap()
        .get_array("firstBatch")
        .unwrap()
        .iter()
        .filter_map(|index| index.as_document())
        .find(|index| index.get_str("name") == Ok("collated_field"))
        .expect("the collated index should have been created")
        .clone();

    let collation = stored
        .get_document("collation")
        .expect("a collated index should report its collation");

    assert!(
        collation.contains_key("version"),
        "expected a server-supplied ICU version; got {collation:?}"
    );
    assert!(
        collation.len() > 2,
        "expected the server to expand the two declared fields; got {collation:?}"
    );
}

/// A declared collation must converge: syncing twice must not rebuild the index. Otherwise every
/// process start rebuilds it, leaving a window where the collection is unindexed, and no caller can
/// declare a collated index that ever settles.
#[tokio::test]
#[ignore]
async fn collated_index_is_not_rebuilt_on_every_sync() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("rust_mongo_orm_tests");

    let collection = CollatedSyncCollConf::collection_name();
    db.collection::<Document>(collection).drop().await.unwrap();

    sync_indexes::<CollatedSyncCollConf>(&db).await.unwrap();
    let after_create = index_stats_since(&db, collection, "collated_field").await;

    sync_indexes::<CollatedSyncCollConf>(&db).await.unwrap();
    let after_second_sync = index_stats_since(&db, collection, "collated_field").await;

    assert_eq!(
        after_create, after_second_sync,
        "the collated index was dropped and recreated by a sync that should have been a no-op"
    );
}
