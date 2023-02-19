default: run

run:
  RUST_LOG=info cargo run -- --sudo-user hobgob -p joxterpwd Cargo.toml joxter@something

test:
  cargo test
