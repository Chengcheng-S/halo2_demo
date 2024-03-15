#!/bin/bash
set -e

cargo check && cargo clippy

cargo run 