# Akri and Krustlet integration
This project contains [Akri](https://github.com/deislabs/akri) components built and designed to be executed as a WebAssembly application inside kubernetes with [Krustlet](https://github.com/deislabs/krustlet).

## About

Akri is a Kubernetes component that finds and lets you use IoT devices under the k8s environment. Krustlet is also a Kubernetes component and lets you run WebAssembly application (In addition to docker containers) on your k8s cluster. This project is about merging these two ideas together and making Akri compiled as a wasm-wasi application be deployed into a Kubernetes cluster using Krustlet, creating new ways to use these devices in your WASM applications and give Akri users all the advantages of this technology.

As it was designed to be runed on the edge, Akri is not expected to consume lots of memory and could benefit from any improvements in this aspect. By making Akri compatible with Krustlet a significant improvement in startup time, runtime performance and memory consumption is expect, reducing the consumption of the user's resources.

## WebAssembly Limitations

### Async functions

For now, Wasm and Wasi don't have a support for asynchronous and multithreading functions, so the discovery handler had to adapted to be strictly synchronously.

### Network components

As mentioned before, Wasm can't handle Async calls, making it challenging to deal with networks. The current state of wasi development doesn’t include a native support for Sockets and Http requests, but the community have created some workarounds that enabled some of these features. These are experimental and not recommended to be used in a production environment but serve as a preview and experience on what using the network on WebAssembly might look like before it is implemented and integrated into the main project.

## Wasi Debug Echo
To create a WebAssembly version of the Debug Echo Discovery Handler and assuming the limitations listed this version of akri has been designed to be executed in a single-threaded environment and only uses files as the communication platform, avoiding dealing with network components.

### How to build and run

This is a WebAssembly project, so its compilation should be targeting the wasm32-wasi architecture.

```
cargo build --target wasm32-wasi --release
```

You can use [wasmtime](https://wasmtime.dev/) to run it locally, since wasm modules are sandboxed, we need to make sure to specify the right storage so it can have access to it during runtime, this project uses `tmp/wde-dir` as the storge directory.

```
wasmtime target/wasm32-wasi/release/wasi-debug-echo.wasm --dir ~/../../tmp/wde-dir 
```

## Discovery Handler gRPC proxy
Since the network components from the Discovery Handler were abstracted this proxy created to execute as a container intermediate the communication between the Akri Agent and the Wasi Debug Echo. This proxy sends to an input file the discovery details sent by the agent so the wasm discovery handler can start discovering the devices, these devices are then received by this proxy in an output file, which is then sent back to the Akri Agent.

### How to build and run

This cargo project can be built and executed using cargo default build process.

```
cargo build --release
cargo run
```
