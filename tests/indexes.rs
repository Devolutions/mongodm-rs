use pretty_assertions::assert_eq;

use mongodb::bson::{Document, doc};
use mongodb::options::ClientOptions;
use mongodb::{Client, Database};
use mongodm::{CollectionConfig, Index, IndexOption, Indexes, sync_indexes};

const TEST_DATABASE: &str = "rust_mongo_orm_tests";
const RESTRICTED_ROLE: &str = "collated_index_manager";
const RESTRICTED_USER: &str = "collated_index_manager";

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
    let db = test_db().await;

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

fn declared_collation() -> Document {
    doc! { "locale": "en", "strength": 2 }
}

fn collated_index() -> Indexes {
    Indexes::new().with(
        Index::new("field")
            .with_option(IndexOption::Name("collated_field".to_owned()))
            .with_option(IndexOption::Collation(declared_collation())),
    )
}

struct CollatedSyncCollConf;

impl CollectionConfig for CollatedSyncCollConf {
    fn collection_name() -> &'static str {
        "collated_sync"
    }

    fn indexes() -> Indexes {
        collated_index()
    }
}

struct SimpleLocaleCollConf;

impl CollectionConfig for SimpleLocaleCollConf {
    fn collection_name() -> &'static str {
        "collated_simple_locale"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new("field")
                .with_option(IndexOption::Name("simple_collated".to_owned()))
                .with_option(IndexOption::Collation(doc! { "locale": "simple" })),
        )
    }
}

struct FrenchCanadianCollConf;

impl CollectionConfig for FrenchCanadianCollConf {
    fn collection_name() -> &'static str {
        "collated_fr_ca"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new("field")
                .with_option(IndexOption::Name("fr_ca_collated".to_owned()))
                .with_option(IndexOption::Collation(doc! { "locale": "fr_CA" })),
        )
    }
}

struct ThaiCollConf;

impl CollectionConfig for ThaiCollConf {
    fn collection_name() -> &'static str {
        "collated_th"
    }

    fn indexes() -> Indexes {
        Indexes::new().with(
            Index::new("field")
                .with_option(IndexOption::Name("th_collated".to_owned()))
                .with_option(IndexOption::Collation(doc! { "locale": "th" })),
        )
    }
}

fn changed_index(collation: Option<Document>) -> Indexes {
    let mut index =
        Index::new("field").with_option(IndexOption::Name("changed_collated".to_owned()));
    if let Some(collation) = collation {
        index = index.with_option(IndexOption::Collation(collation));
    }

    Indexes::new().with(index)
}

struct UncollatedCollConf;

impl CollectionConfig for UncollatedCollConf {
    fn collection_name() -> &'static str {
        "collated_changed"
    }

    fn indexes() -> Indexes {
        changed_index(None)
    }
}

struct CollationAddedCollConf;

impl CollectionConfig for CollationAddedCollConf {
    fn collection_name() -> &'static str {
        "collated_changed"
    }

    fn indexes() -> Indexes {
        changed_index(Some(doc! { "locale": "en" }))
    }
}

struct CollationRelocaledCollConf;

impl CollectionConfig for CollationRelocaledCollConf {
    fn collection_name() -> &'static str {
        "collated_changed"
    }

    fn indexes() -> Indexes {
        changed_index(Some(doc! { "locale": "fr_CA" }))
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
        if stats.get_str("name").is_ok_and(|name| name == index_name) {
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

async fn test_db() -> mongodb::Database {
    let uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_owned());
    let client_options = ClientOptions::parse(uri).await.unwrap();

    Client::with_options(client_options)
        .unwrap()
        .database(TEST_DATABASE)
}

async fn restricted_test_db() -> mongodb::Database {
    let uri = std::env::var("MONGODM_RESTRICTED_MONGODB_URI")
        .expect("the restricted-role integration test requires MONGODM_RESTRICTED_MONGODB_URI");
    let client_options = ClientOptions::parse(uri).await.unwrap();

    Client::with_options(client_options)
        .unwrap()
        .database(TEST_DATABASE)
}

async fn drop_restricted_user_and_role(db: &mongodb::Database) {
    let users = db
        .run_command(doc! {
            "usersInfo": { "user": RESTRICTED_USER, "db": TEST_DATABASE },
        })
        .await
        .unwrap()
        .get_array("users")
        .unwrap()
        .to_owned();
    if !users.is_empty() {
        db.run_command(doc! { "dropUser": RESTRICTED_USER })
            .await
            .unwrap();
    }

    let roles = db
        .run_command(doc! {
            "rolesInfo": { "role": RESTRICTED_ROLE, "db": TEST_DATABASE },
        })
        .await
        .unwrap()
        .get_array("roles")
        .unwrap()
        .to_owned();
    if !roles.is_empty() {
        db.run_command(doc! { "dropRole": RESTRICTED_ROLE })
            .await
            .unwrap();
    }
}

/// The `listIndexes` entry for one index, as the server reports it.
async fn listed_index(db: &mongodb::Database, collection: &str, index_name: &str) -> Document {
    let ret = db
        .run_command(doc! { "listIndexes": collection })
        .await
        .unwrap();

    ret.get_document("cursor")
        .unwrap()
        .get_array("firstBatch")
        .unwrap()
        .iter()
        .filter_map(|index| index.as_document())
        .find(|index| index.get_str("name").is_ok_and(|name| name == index_name))
        .unwrap_or_else(|| panic!("index `{index_name}` should have been created"))
        .clone()
}

/// Creates and resyncs an index, asserting that `accesses.since` is unchanged.
async fn assert_index_survives_a_resync<Conf: CollectionConfig>(
    index_name: &str,
) -> mongodb::Database {
    let db = test_db().await;
    let collection = Conf::collection_name();

    db.collection::<Document>(collection).drop().await.unwrap();

    sync_indexes::<Conf>(&db).await.unwrap();
    let after_create = index_stats_since(&db, collection, index_name).await;

    sync_indexes::<Conf>(&db).await.unwrap();
    let after_resync = index_stats_since(&db, collection, index_name).await;

    assert_eq!(
        after_create, after_resync,
        "`{collection}`: the index was dropped and recreated by a sync that should have been a no-op"
    );

    db
}

#[tokio::test]
#[ignore]
async fn a_collated_index_converges_with_and_without_find_privilege() {
    let db = assert_index_survives_a_resync::<CollatedSyncCollConf>("collated_field").await;
    let collection = CollatedSyncCollConf::collection_name();
    let stored = listed_index(&db, collection, "collated_field").await;
    let collation = stored
        .get_document("collation")
        .expect("a collated index should report its collation");

    assert!(collation.contains_key("version"));
    assert!(collation.len() > declared_collation().len());

    drop_restricted_user_and_role(&db).await;
    db.run_command(doc! {
        "createRole": RESTRICTED_ROLE,
        "privileges": [{
            "resource": { "db": TEST_DATABASE, "collection": collection },
            "actions": ["listIndexes", "createIndex", "dropIndex"],
        }],
        "roles": [],
    })
    .await
    .unwrap();
    db.run_command(doc! {
        "createUser": RESTRICTED_USER,
        "pwd": "password",
        "roles": [{ "role": RESTRICTED_ROLE, "db": TEST_DATABASE }],
    })
    .await
    .unwrap();

    let before_restricted_sync = index_stats_since(&db, collection, "collated_field").await;
    let restricted_db = restricted_test_db().await;
    sync_indexes::<CollatedSyncCollConf>(&restricted_db)
        .await
        .unwrap();
    let after_restricted_sync = index_stats_since(&db, collection, "collated_field").await;

    assert_ne!(
        before_restricted_sync, after_restricted_sync,
        "a user without find must recreate the collated index"
    );

    drop_restricted_user_and_role(&db).await;
}

#[tokio::test]
#[ignore]
async fn a_simple_locale_index_survives_a_resync() {
    let db = assert_index_survives_a_resync::<SimpleLocaleCollConf>("simple_collated").await;

    let stored = listed_index(
        &db,
        SimpleLocaleCollConf::collection_name(),
        "simple_collated",
    )
    .await;

    assert!(
        !stored.contains_key("collation"),
        "the simple locale should be stored as no collation at all; got {stored:?}"
    );
}

#[tokio::test]
#[ignore]
async fn a_french_canadian_index_survives_a_resync() {
    assert_index_survives_a_resync::<FrenchCanadianCollConf>("fr_ca_collated").await;
}

#[tokio::test]
#[ignore]
async fn a_thai_index_survives_a_resync() {
    assert_index_survives_a_resync::<ThaiCollConf>("th_collated").await;
}

/// A changed collation declaration must recreate the index.
#[tokio::test]
#[ignore]
async fn changing_the_declared_collation_rebuilds_the_index() {
    let db = test_db().await;
    let collection = UncollatedCollConf::collection_name();

    db.collection::<Document>(collection).drop().await.unwrap();

    sync_indexes::<UncollatedCollConf>(&db).await.unwrap();
    let uncollated = index_stats_since(&db, collection, "changed_collated").await;

    sync_indexes::<CollationAddedCollConf>(&db).await.unwrap();
    let collated = index_stats_since(&db, collection, "changed_collated").await;

    assert_ne!(
        uncollated, collated,
        "adding a collation should have rebuilt the index, but it was left uncollated"
    );

    sync_indexes::<CollationRelocaledCollConf>(&db)
        .await
        .unwrap();
    let relocaled = index_stats_since(&db, collection, "changed_collated").await;

    assert_ne!(
        collated, relocaled,
        "changing the locale should have rebuilt the index, but it was left alone"
    );

    let stored = listed_index(&db, collection, "changed_collated").await;
    let locale = stored
        .get_document("collation")
        .expect("the rebuilt index should be collated")
        .get_str("locale")
        .expect("a collation should report its locale");

    assert_eq!(
        locale, "fr_CA",
        "the rebuilt index should carry the newly declared locale"
    );
}
