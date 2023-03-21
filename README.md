# Beyond HTTP Microservices with Spin

Slides and samples for this Wasm I/O talk.

## Running the Sample

Dependencies:

* [Install Spin on your PATH](https://developer.fermyon.com/spin/install)
* [Install Rust](https://rustup.rs/)
* Install the Rust WASI backend: `rustup target add wasm32-wasi`

Set up Spin plugins:

* `make prereqs`

Build and run the trigger and demo app:

* `make test`

Other make targets:

* `make build`: build the trigger binary only
* `make install`: build the trigger plugin and install into Spin
* `make guest`: build the guest Wasm module only
