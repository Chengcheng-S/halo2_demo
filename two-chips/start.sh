#!/bin/bash
set -e

if ! command -v rustc &> /dev/null
then
    echo "Rust is not installed, installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source ~/.bashrc
else
    echo "Rust is already installed"
fi


cargo check && cargo clippy

sleep 5 

cargo run 

sleep 5

echo "uninstall rust env"
rustup self uninstall
source ~/.bashrc
echo "plase restart your terminal && run rustc -V or cargo -V check the rust uninstall"