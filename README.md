# Akri and Krustlet integration
This project contains [Akri](https://github.com/deislabs/akri) components built and designed to be executed as a [WebAssembly](https://webassembly.org/) application inside Kubernetes with [Krustlet](https://github.com/deislabs/krustlet).

## About

Akri is a Kubernetes component that finds and lets you use IoT devices under the k8s environment. Krustlet is also a Kubernetes component and lets you run WebAssembly application (In addition to docker containers) on your k8s cluster. This project is about merging these two ideas together and making Akri compiled as a wasm-wasi application be deployed into a Kubernetes cluster using Krustlet, creating new ways to use these devices in your Wasm applications and give Akri users all the advantages of this technology.

As it was designed to be executed on the edge, Akri is not expected to consume lots of memory and could benefit from any improvements in this aspect. By making Akri compatible with Krustlet a significant improvement in startup time, runtime performance and memory consumption is expected, reducing the consumption of the user's resources.

### WebAssembly (Wasm)

Wasm has been a promising tool and has already revolutionized web development, but all the predictability, scalability, efficiency, and security improvements that it has when compared with other browsers tools like JavaScript could also be applied to server-side technologies like servers and services. 

[WebAssembly Systems Interface (Wasi)](https://wasi.dev/) is part of the movement to use WebAssembly in the server and our goal is to put this into practice and bring it to the Kubernetes environment and enrich the Krustlet ecosystem, taking advantage from the “compiled once, and run anywhere” lemma.

## WebAssembly Limitations

### Async functions

For now, Wasm and Wasi don't have a support for asynchronous and multithreading functions, so the discovery handler had to adapted to be strictly synchronously.

### Network components

As mentioned before, Wasm can't handle Async calls, making it challenging to deal with networks. The current state of Wasi development doesn’t include native support for Sockets and Http requests, but the community have created some workarounds that enabled some of these features. These are experimental and not recommended to be used in a production environment but serve as a preview and experience on what using the network on WebAssembly might look like before it is implemented and integrated into the main project.

## Wasi Debug Echo
Debug Echo is a Akri Discovery Handler for debugging and serves to test Akri devices, more details can be found [here](https://github.com/deislabs/akri/blob/main/docs/debug-echo-configuration.md)

To create a WebAssembly version of the Debug Echo Discovery Handler and assuming the limitations listed this version of Akri has been designed to be executed in a single-threaded environment and only uses files as the communication platform, avoiding dealing with network components.

### How to build and run

This is a WebAssembly project, so its compilation should be targeting the wasm32-wasi architecture.

```
cargo build --target wasm32-wasi --release
```

You can use [wasmtime](https://wasmtime.dev/) to run it locally, since Wasm modules are sandboxed, we need to make sure to specify the right storage so it can have access to it during runtime, this project uses `tmp/wde-dir` as the storage directory.

```
wasmtime target/wasm32-wasi/release/wasi-debug-echo.wasm --dir ~/../../tmp/wde-dir 
```

The [wasm-to-oci](https://github.com/engineerd/wasm-to-oci) project needs to be used in order To deploy this wasm application to a container registry of your choice. This can later be used to pull the project image into a Kubernetes deployment.
> Keep in mind that Docker Hub currently doesn’t support these types of images to their registry as it uses an unknown format, so another provider is recommended.

```
wasm-to-oci push target/wasm32-wasi/release/wasi-debug-echo.wasm {YOUR_OCI_REGISTRY}.
```

## Discovery Handler gRPC proxy
Since the network components from the Discovery Handler were abstracted, this proxy created to execute as a container intermediate the communication between the Akri Agent and any WebAssembly Discovery Handles. This proxy sends to an input file the discovery details sent by the agent so the wasm Discovery Handler can start discovering the devices, these devices are then received by this proxy in an output file, which is then sent back to the Akri Agent.

### How to build and run

This cargo project can be built and executed using cargo default build process.
You can set the Discovery Handler name with the `DISCOVERY_HANDLER_NAME` environment variable, if not specified `debugEcho` will be used.

```
cargo build --release
cargo run
```

## Demos

### Krustlet using Akri devices
Try the [Krustlet using Akri devices demo](./demo-kubelet.md) to test and check out the usage of devices discovered by a WebAssembly Discovery Handler by a Wasm Module running on Krustlet. The result will be the architecture below.

<img src="./KrustletUsingAkriDevicesDesign.png" alt="Krustlet integration architecture" style="padding-bottom: 10px padding-top: 10px;
margin-right: auto; display: block; margin-left: auto;"/>

### Akri using devices discovered by a Wasi Discovery Handler
Try the [Krustlet and Akri Integration demo](./demo-kubelet.md) to test and check out the usage of devices discovered by a WebAssembly Discovery Handler into a regular Akri cluster. The result will be the architecture below.

<img src="./AkriUsingKrustletDevicesDesign.png" alt="Krustlet integration architecture" style="padding-bottom: 10px padding-top: 10px;
margin-right: auto; display: block; margin-left: auto;"/>
