#!/bin/bash

# Build for all architectures
cargo build --release || exit 1
cross build --target x86_64-pc-windows-gnu --release || exit 1
cross build --target x86_64-apple-darwin --release || exit 1
cross build --target aarch64-apple-darwin --release || exit 1

rm -rf target/final || exit 1
mkdir target/final

# Copy the binaries to the final directory
cp target/release/seneca-solver target/final/seneca-solver_linux-x86_64 || exit 1
cp target/x86_64-pc-windows-gnu/release/seneca-solver.exe target/final/seneca-solver_windows-x86_64.exe || exit 1
cp target/x86_64-apple-darwin/release/seneca-solver target/final/seneca-solver_macos-x86_64 || exit 1
cp target/aarch64-apple-darwin/release/seneca-solver target/final/seneca-solver_macos-aarch64 || exit 1
