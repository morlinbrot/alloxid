# alloxid-http
Backend of the `alloxid` family of crates, featuring JWT authentication and a dockerized database.

## Usage
With [`docker`](https://www.docker.com/) and [`sqlx-cli`](https://crates.io/crates/sqlx-cli) being installed, run:
```
./scripts/init_db.sh
cargo run
```
Open your browser and navigate to `localhost:3000/health-check`.

A set of integration tests can be found in the [`tests`](/tests) folder. Use [`cargo nextest`](https://nexte.st/) for a modern test experience.
