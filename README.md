# Akri and Krustlet integration
WebAssembly Systems Interface (Wasi) version for akri.

## About

Akri is a Kubernetes component that finds and lets you use IoT devices under the k8s environment. Krustlet is also a Kubernetes component and lets you run WebAssembly application (In addition to docker containers) on your k8s cluster. What we are doing is merging these two ideas together and making Akri compiled as a wasm-wasi application be deployed into a Kubernetes cluster using Krustlet, creating new ways to use these devices in your WASM applications and give Akri users all the advantages of this technology.

As it was designed to be runed on the edge, Akri is not expected to consume lots of memory and could benefit from any improvements in this aspect. By making Akri compatible with Krustlet we expect a significant improvement in startup time, runtime performance and memory consumption, reducing the consumption of the user's resources.

## WebAssembly Limitations

### Async functions

For now, Wasm and Wasi don't have a support for asynchronous and multithreading functions, so we had to adapt the discovery handler to be strictly synchronously.

### Network components

As mentioned before, Wasm can't handle Async calls, making it challenging to deal with networks. The current state of wasi development doesn’t include a native support for Sockets and Http requests, but the community have created some workarounds that enabled some of these features. These are experimental and not recommended to be used in a production environment but serve as a preview and experience on what using the network on WebAssembly might look like before it is implemented and integrated into the main project.

## Wasi Debug Echo
To create a WebAssembly version of the Debug Echo Discovery Handler and assuming the limitations listed this version has been designed to be executed in a single-threaded environment and only uses files as the communication platform, avoiding dealing with network components.

## Wasi Debug Echo (Wde) gRPC proxy
Since we abstracted the network components from the Discovery Handler this proxy builded to executed as a container has been created to intermediate the communication between the Akri's Agent and the Wasi Debug Echo. This proxy sends to an input file everything the agents inputs and sends to the agent everything printed to the output file.
