start *ARGS:
  RUST_LOG=lattice=info cargo run --release {{ARGS}}

debug *ARGS:
  RUST_LOG=lattice=debug cargo run --release {{ARGS}}

trace *ARGS:
  RUST_LOG=lattice=trace cargo run --release {{ARGS}}

# Usage: just trace-module framework::frame_controller
trace-module MODULE *ARGS:
    RUST_LOG=lattice=info,lattice::{{MODULE}}=trace cargo run --release {{ARGS}}

test *ARGS:
  RUST_LOG=lattice=trace cargo test -- {{ARGS}}

test-1-thread *ARGS:
  RUST_LOG=lattice=trace cargo test -- --test-threads=1 {{ARGS}}