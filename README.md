## alloxid
All is oxidized!

A full stack web application in Rust, featuring JWT authentication and a dockerized database.

## Usage
With [`docker`](https://www.docker.com/) and [`sqlx-cli`](https://crates.io/crates/sqlx-cli) being installed, run:
```
./scripts/init_db.sh
cargo run
```
Open your browser and navigate to `localhost:8080/health-check`.

The program uses the `dotenv` crate to read the two env vars and expects a `DATABASE_URL` and `APP_SECRET` to be set. Consider using a .env file in the root for this.

A set of integration tests can be found in the [`tests`](/tests) folder. Use [`cargo nextest`](https://nexte.st/) for a modern test experience.

## Debugging
Enter the container and have a look at the database (-it - interactive tty):
```
docker exec -it alloxid_db bash
psql -U postgres -d alloxid
```

## Prior Art
https://www.youtube.com/watch?v=yNe9Xr35n4Q \
https://www.lpalmieri.com/posts/2020-06-06-zero-to-production-1-setup-toolchain-ides-ci/ \
https://github.com/colinbankier/realworld-tide

