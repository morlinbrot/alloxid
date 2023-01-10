default:
  just --list --unsorted

# Run clean, init, http in sequence.
run:
  just clean && just init && just http

# Stop and remove docker container
clean:
  docker stop alloxid_db
  docker rm alloxid_db

# Set up docker image, create db and run migrations
init:
  ./scripts/init_db.sh

# Run all tests
test *PARAMS:
  cargo nextest run {{PARAMS}}

watch *PARAMS:
  cargo watch -s "just test" {{PARAMS}}

# Run the http server
http *PARAMS:
  cargo run {{PARAMS}}

front *PARAMS:
  cd alloxid-front; dioxus serve

# Run clippy
clip *PARAMS:
  cargo clippy {{PARAMS}}
