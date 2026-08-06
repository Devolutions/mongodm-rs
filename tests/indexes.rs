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

/// Shared so the expansion test can assert the server returned more fields than were declared,
/// without hard-coding how many that is.
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

struct CollatedExpansionCollConf;

impl CollectionConfig for CollatedExpansionCollConf {
    fn collection_name() -> &'static str {
        "collated_expansion"
    }

    fn indexes() -> Indexes {
        collated_index()
    }
}

// Locales whose server-side defaults differ from `en`: fr_CA flips `backwards`, da sets `caseFirst`
// to "upper", th sets `alternate` to "shifted" and `normalization` to true. Any implementation that
// guesses defaults from a table rather than asking the server rebuilds these on every sync.
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

// Three declarations of the same index on one collection: uncollated, then collated, then collated
// for a different locale. Syncing them in order is two genuine changes, each of which has to rebuild.
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
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();

    Client::with_options(client_options)
        .unwrap()
        .database("rust_mongo_orm_tests")
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

/// Syncs from scratch, then syncs again and asserts the index survived the second, no-op call.
///
/// `accesses.since` is reset by a recreate, so an unchanged value is what proves the declaration
/// converged. Each locale below runs the same steps, which is the point — the comparison must not
/// care which locale it is looking at. Returns the database so a caller can assert further without
/// opening a second client.
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

/// Documents the server behaviour the convergence tests rest on, kept apart from them so a failure
/// says which of the two broke: the server changing how it reports collations, or `sync_indexes`
/// mishandling what it reports.
#[tokio::test]
#[ignore]
async fn the_server_expands_a_declared_collation() {
    let db = test_db().await;
    let collection = CollatedExpansionCollConf::collection_name();

    db.collection::<Document>(collection).drop().await.unwrap();
    sync_indexes::<CollatedExpansionCollConf>(&db)
        .await
        .unwrap();

    let stored = listed_index(&db, collection, "collated_field").await;
    let collation = stored
        .get_document("collation")
        .expect("a collated index should report its collation");

    assert!(
        collation.contains_key("version"),
        "expected a server-supplied ICU version; got {collation:?}"
    );
    assert!(
        collation.len() > declared_collation().len(),
        "expected the server to expand {} declared fields; got {collation:?}",
        declared_collation().len()
    );
}

#[tokio::test]
#[ignore]
async fn a_collated_index_survives_a_resync() {
    assert_index_survives_a_resync::<CollatedSyncCollConf>("collated_field").await;
}

/// `locale: "simple"` converges only because the server stores no collation for it, which is asserted
/// here rather than in the expansion test so this collection stays owned by one test.
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

/// `fr_CA` expands with `backwards: true`, unlike `en`.
#[tokio::test]
#[ignore]
async fn a_french_canadian_index_survives_a_resync() {
    assert_index_survives_a_resync::<FrenchCanadianCollConf>("fr_ca_collated").await;
}

/// `th` expands with `alternate: "shifted"` and `normalization: true`, two more deviations from `en`.
#[tokio::test]
#[ignore]
async fn a_thai_index_survives_a_resync() {
    assert_index_survives_a_resync::<ThaiCollConf>("th_collated").await;
}

/// The inverse of every test above: a declaration that really changed has to rebuild.
///
/// Adding a collation to an uncollated index is the step that catches a comparison unable to read the
/// server's expansion. Treating an unreadable expansion as "no collation" leaves both sides without
/// one, they compare equal, and the sync reports success while the index stays uncollated. Changing
/// the locale afterwards then checks the rebuild lands the declaration rather than just happening.
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
