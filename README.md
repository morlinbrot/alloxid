## fullstack-rs
A full stack web application in Rust, featuring JWT authentication and a dockerized database.

## Usage
With `docker` and `sqlx-cli` being installed, run:
```
./scripts/init_db.sh
cargo run
```

Enter the container and have a look at the database (-it - interactive tty):
```
docker exec -it fullstack_db bash
psql -U postgres -d fullstack
```

The program uses the `dotenv` crate to read the two env vars `DATABASE_URL` and
`APP_SECRET` from a .env file.

## Debugging
To enter the docker container and have a look at the databases living inside:
```
docker exec -it fullstack_db bash

psql -U postgres
```

## Notes
~~It's not possible to run all tests at the same time (`cargo test`) because of some kind
of concurrency issue. Measures that have been taken to check the database for any
open connections before tearing it down don't seem to work properly, which is why the
tests will fail sometimes when a thread tries to tear down the database while a connection
is still active. Tests can be run individually though.~~  
EDIT: Problem might have been a different one. The introduction of a short timeout during
TestApp setup let's us run all tests at the same time.

## Inspiration
https://www.youtube.com/watch?v=yNe9Xr35n4Q \
https://www.lpalmieri.com/posts/2020-06-06-zero-to-production-1-setup-toolchain-ides-ci/ \
https://github.com/colinbankier/realworld-tide

