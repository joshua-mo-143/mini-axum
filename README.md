# mini-axum: An MVP version of Axum.
This is `mini-axum`, a crate that aims to be a simpler and more understandable version of the official `axum` crate. Follows on from [TODO] where we talk about Axum in-depth and discuss some of its core components, and then try to re-create it.

## Features
- Spin up a server (using `hyper` under the hood)
- Allow different handlers depending on request method
- Take requests and use extractors to parse data from a request in handler functions
- Allow shared state in handlers
- Use middleware

## Usage
Run `cargo run --example basic` to run the basic example. Alternatively, peruse the codebase!
