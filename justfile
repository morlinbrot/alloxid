default:
  just --list --unsorted

# Stop and remove docker container
clean:
  docker stop alloxid_db
  docker rm alloxid_db

# Set up docker image, create db and run migrations
init:
  just clean
  ./scripts/init_db.sh

# Run the specified crate
run crate="http":
  just {{crate}}

set positional-arguments

# Run tests for the specified crate
test *crate:
  #!/usr/bin/env sh
  set -euxo pipefail
  crate=""
  # if the number of args is greater than 0
  if [ $# -gt 0 ]; then
    if [ $1 == "all" ]; then
      crate="-p alloxid-http -p alloxid-grpc -p alloxid-front"
    else
      crate="-p alloxid-$1"
    fi
  fi
  cargo nextest run $crate

# cargo-watch another just recipe
watch cmd *PARAMS:
  cargo watch -s "just {{cmd}} {{PARAMS}}"

# Run the http server
http *PARAMS:
  cargo run {{PARAMS}}

# Run the Dioxus frontend
front *PARAMS:
  cd alloxid-front; dioxus serve

# Run the gRPC server
grpc *PARAMS:
  cd alloxid-grpc; cargo run {{PARAMS}}
