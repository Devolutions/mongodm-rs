#[macro_use]
extern crate pretty_assertions;

use futures_util::StreamExt;
use mongodm::prelude::*;
use serde::{Deserialize, Serialize};

struct UserCollConf;

impl CollectionConfig for UserCollConf {
    fn collection_name() -> &'static str {
        "some_operations"
    }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new(f!(name in User)).with_option(IndexOption::Unique))
            .with(Index::new(f!(age in User)))
    }
}

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    age: i32,
    info: String,
}

impl Model for User {
    type CollConf = UserCollConf;
}

#[tokio::test]
#[ignore]
async fn insert_delete_find() {
    let client_options = MongoClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = MongoClient::with_options(client_options).unwrap();
    let db = client.database("rust_mongo_orm_tests");

    let repository = db.repository::<User>();
    repository.drop(None).await.unwrap();
    sync_indexes::<UserCollConf>(&db).await.unwrap();

    let users = vec![
        User {
            name: String::from("David"),
            age: 35,
            info: String::from("a"),
        },
        User {
            name: String::from("Stacey"),
            age: 20,
            info: String::from("b"),
        },
        User {
            name: String::from("Danniella"),
            age: 18,
            info: String::from("c"),
        },
        User {
            name: String::from("Dane"),
            age: 47,
            info: String::from("d"),
        },
        User {
            name: String::from("Teri"),
            age: 82,
            info: String::from("e"),
        },
        User {
            name: String::from("Edna"),
            age: 57,
            info: String::from("f"),
        },
        User {
            name: String::from("Reeva"),
            age: 39,
            info: String::from("g"),
        },
    ];

    repository.insert_many(users, None).await.unwrap();

    let user_dane = repository
        .find_one(doc! { f!(name in User): "Dane" }, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_dane.name, "Dane");
    assert_eq!(user_dane.age, 47);
    assert_eq!(user_dane.info, "d");

    let found = repository
        .find(doc! { f!(age in User): { LesserThan: 40 } }, None)
        .await
        .unwrap();
    let found: Vec<mongodb::error::Result<User>> = found.collect().await;
    assert_eq!(found.len(), 4);

    repository
        .delete_one(doc! { f!(age in User): { LesserThan: 38 } }, None)
        .await
        .unwrap();

    let found = repository
        .find(doc! { f!(age in User): { LesserThan: 40 } }, None)
        .await
        .unwrap();
    let found: Vec<mongodb::error::Result<User>> = found.collect().await;
    assert_eq!(found.len(), 3);
}

#[tokio::test]
#[ignore]
async fn bulk_updates() {
    let client_options = MongoClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = MongoClient::with_options(client_options).unwrap();
    let db = client.database("rust_mongo_orm_tests");

    let repository = db.repository::<User>();
    repository.drop(None).await.unwrap();
    sync_indexes::<UserCollConf>(&db).await.unwrap();

    let users = vec![
        User {
            name: String::from("David"),
            age: 35,
            info: String::from("a"),
        },
        User {
            name: String::from("Stacey"),
            age: 20,
            info: String::from("b"),
        },
        User {
            name: String::from("Danniella"),
            age: 18,
            info: String::from("c"),
        },
        User {
            name: String::from("Dane"),
            age: 47,
            info: String::from("d"),
        },
        User {
            name: String::from("Teri"),
            age: 82,
            info: String::from("e"),
        },
        User {
            name: String::from("Edna"),
            age: 57,
            info: String::from("f"),
        },
        User {
            name: String::from("Reeva"),
            age: 39,
            info: String::from("g"),
        },
    ];

    repository.insert_many(users, None).await.unwrap();

    let bulk_update_res = repository
        .bulk_update(&vec![
            &BulkUpdate {
                query: doc! { f!(name in User): "Dane" },
                update: doc! { Set: { f!(age in User): 12 } },
                options: None,
            },
            &BulkUpdate {
                query: doc! { f!(name in User): "David" },
                update: doc! { Set: { f!(age in User): 30 } },
                options: None,
            },
        ])
        .await
        .unwrap();
    assert_eq!(bulk_update_res.nb_affected, 2);
    assert_eq!(bulk_update_res.nb_modified, 2);

    let user_dane = repository
        .find_one(doc! { f!(name in User): "Dane" }, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_dane.name, "Dane");
    assert_eq!(user_dane.age, 12);

    let user_dane = repository
        .find_one(doc! { f!(name in User): "David" }, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_dane.name, "David");
    assert_eq!(user_dane.age, 30);
}
