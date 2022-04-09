# alloxid-http
Backend of the `alloxid` family of crates, featuring JWT authentication and a dockerized database.

## Usage
With [`docker`](https://www.docker.com/) and [`sqlx-cli`](https://crates.io/crates/sqlx-cli) being installed, run:
```
./scripts/init_db.sh
cargo run
```
Open your browser and navigate to `localhost:3000/health-check`.

The program uses the `dotenv` crate to read env vars and expects a `DATABASE_URL`, `DATABASE_PASSWORD` and `APP_SECRET` to be set. Consider using a .env file in the root for this. Example:
```
export DATABASE_URL="postgres://postgres:password@localhost:54321/alloxid"
export DATABASE_PASSWORD="password"
export APP_SECRET="password"
```

A set of integration tests can be found in the [`tests`](/tests) folder. Use [`cargo nextest`](https://nexte.st/) for a modern test experience.

## Debugging
Enter the container and have a look at the database (-it - interactive tty):
```
docker exec -it alloxid_db bash
psql -U postgres -d alloxid
```
