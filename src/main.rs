// src/main.rs

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "web")]
mod web;

#[cfg(feature = "cli")]
fn main() {
    cli::main().unwrap();
}

#[cfg(feature = "web")]
fn main() {
    web::main().unwrap();
}
