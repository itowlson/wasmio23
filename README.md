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

Exercise the demo app (after `make test` or `spin up -f guest`):

* `telnet localhost 8089` and type a line of text followed by Enter

```
$ telnet localhost 8089
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
hello wasmio
¡HOLA FROM BARCELONA!
Connection closed by foreign host.
```
