# alloxid
'Tis all oxidized!

Family of crates for a cloud-native microservices stack written fully in Rust featuring
- A backend made with [axum](https://github.com/tokio-rs/axum) with JWT authentication and a dockerized database
- A frontend made with [Dioxus](https://github.com/dioxuslabs/dioxus)
- A collection of gRPC services made with [Tonic](https://github.com/hyperium/tonic)

## Prerequisites
- [`Docker`](https://www.docker.com/)
- [`sqlx-cli`](https://crates.io/crates/sqlx-cli)
- [`dioxus-cli`](https://crates.io/crates/dioxus-cli)

The app uses the `dotenv` crate to read env vars and expects a `DATABASE_URL`, `DATABASE_PASSWORD` and `APP_SECRET` to be set. Consider using a .env file in the root for this.

## Usage
This project uses the awesome [just command runner](https://github.com/casey/just). Run `just` in the root folder to see the available commands.

Quickstart:
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
See the individual crates' READMEs for additional details.  

## Debugging
Enter the container and have a look at the database (-it - interactive tty):
```
docker exec -it alloxid_db bash
psql -U postgres -d alloxid
```
