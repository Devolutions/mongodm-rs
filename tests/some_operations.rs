#[macro_use]
extern crate pretty_assertions;

use futures_util::StreamExt;
use mongodb::{bson::doc, options::ClientOptions, Client};
use mongodm::operator::*;
use mongodm::{f, DatabaseConfig, DatabaseConfigExt, Index, IndexOption, Indexes, Model};
use serde::{Deserialize, Serialize};

struct TestDb;

impl DatabaseConfig for TestDb {
    fn db_name() -> &'static str {
        "rust_mongo_orm_tests"
    }
}

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    age: i32,
    info: String,
}

impl Model for User {
    fn coll_name() -> &'static str {
        "some_operations"
    }

    fn indexes() -> Indexes {
        Indexes::new()
            .with(Index::new("name").with_option(IndexOption::Unique))
            .with(Index::new("age"))
    }
}

#[tokio::test]
#[ignore]
async fn insert_delete_find() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();

    let repository = TestDb::get_repository::<User>(client);
    repository.drop(None).await.unwrap();
    repository.sync_indexes().await.unwrap();

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
