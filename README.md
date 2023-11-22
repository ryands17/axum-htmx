# Axum + HTMX

This is a simple Todos API made in Rust using Axum and a fully server rendered UI with HTMX. A blog post will soon follow

## Commands

Just run `cargo run` in the terminal or use [cargo-watch](https://crates.io/crates/cargo-watch) to watch for changes. It will watch for changes in the `src` and `templates` folder for changes to the API or HTML respectively.

To install `cargo-watch`:

```sh
cargo install cargo-watch
```

To run the server in watch mode:

```sh
cargo watch -c -w src -w templates -x run
```
