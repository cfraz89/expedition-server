set dotenv-load

dev:
  RUST_LOG=debug cargo watch -x run
  
surreal:
  surreal start --log debug --user root --pass root memory