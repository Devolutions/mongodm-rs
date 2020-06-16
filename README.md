MongODM
=======

A thin ODM layer for MongoDB built upon the [official Rust driver](https://github.com/mongodb/mongo-rust-driver).

Main features:

- A stronger API leveraging Rust type system
- Data structure models are defined using the well-known [`serde`](https://github.com/serde-rs/serde) serialization framework
- Index support on top of the `Database::run_command` (index management is currently not implemented in the underlying driver)
- Indexes synchronization
- Additional compile-time checks for queries using macros and type associated to mongo operators (eg: `And` instead of "$and")

See documentation for examples.

## Tests

Some tests can be run with `cargo test` however most of the useful tests requires a Mongo database running and exposed on `localhost:27017`.
These integration tests are run with `cargo test -- --ignored`.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
