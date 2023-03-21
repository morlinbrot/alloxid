# alloxid
'Tis all oxidized!

Family of crates for a cloud-native microservices stack written mainly in Rust featuring
- A backend made with [axum](https://github.com/tokio-rs/axum) with JWT authentication and a dockerized database
- A frontend made with [Deno's Fresh](https://github.com/denoland/fresh)
- A collection of gRPC services made with [Tonic](https://github.com/hyperium/tonic)

## Prerequisites
- [`Docker`](https://www.docker.com/)
- [`sqlx-cli`](https://crates.io/crates/sqlx-cli)

The app uses the `dotenv` crate to read env vars and expects a `DATABASE_URL`, `DATABASE_PASSWORD` and `APP_SECRET` to be set. Consider using a .env file in the root for this.

## Usage
This project uses the awesome [just command runner](https://github.com/casey/just). Run `just` in the root folder to see the available commands, and `just -s COMMAND` to see what a command does.

### Quickstart
See below for setting up for the first time.

In root dir:
```
just run
```
In another terminal:
```
just front
```
And yet another terminal:
```
just grpc
```
Some of the commands can be chained and applied to a specific crate, e.g.:
```
just watch test grpc
```
Will `cargo-watch` running `nextest` in the `alloxid-grpc` crate.

### First start
With [`docker`](https://www.docker.com/) and [`sqlx-cli`](https://crates.io/crates/sqlx-cli) being installed, run
```
just init
```
to set up the database inside a Docker container.

For a full reset (this will wipe the database!) run:
```
just clean
```

## Debugging
Enter the container and have a look at the database (-it - interactive tty):
```
docker exec -it alloxid_db bash
psql -U postgres -d alloxid
```
