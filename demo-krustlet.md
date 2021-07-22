# Akri and Krustlet integration demo
This is a demo to showcase the usage of Akri discovered devices into a WebAssembly application running on Krustlet.

The architecture we are achieving at the end is shown above, a local running Akri Agent will communicate with Krustlet device plugin and allow the creation of the resources discovered by the Wasi Discovery Handler.

<img src="./KrustletUsingAkriDevicesDesign.png" alt="Krustlet integration architecture" style="padding-bottom: 10px padding-top: 10px;
margin-right: auto; display: block; margin-left: auto;"/>

## Start your kubernetes cluster

For this demo we are using microk8s, but feel free to use any of your choice, krustlet has documentation for most of them [here](https://github.com/deislabs/krustlet/tree/main/docs/howto).

## Start your krustlet node

For this demo we are using an unreleased version of krustle that enable the device plugin features. So we are running krustlet directily from the main branch from GitHub using the command:

```
KRUSTLET_NODE_IP=127.0.13.1 \
	KRUSTLET_HOSTNAME=krustlet \
	KRUSTLET_NODE_NAME=krustlet \
	KRUSTLET_BOOTSTRAP_FILE={YOUR_BOOTSTRAP_FILE} \
	KRUSTLET_DEVICE_PLUGINS_DIR=~/device-plugins/ \
	just run
```
> Make sure the Node ip informed is present on the known hosts list (Manually add it if not present).

## Start Akri Agent

To inform the krustlet node about new resources and communication with the Discovery Handler we will now run the Akri Agent from Akri main branch. 

```
kubectl apply -f ./deployment/helm/crds
cargo build --release
RUST_LOG=info RUST_BACKTRACE=1 KUBECONFIG=~/.kube/config \
	DISCOVERY_HANDLERS_DIRECTORY=~/device-plugins/ \
	AGENT_NODE_NAME=krustlet \
	HOST_CRICTL_PATH=/usr/local/bin/crictl \
	HOST_RUNTIME_ENDPOINT=/var/snap/microk8s/common/run/containerd.sock \
	HOST_IMAGE_ENDPOINT=/var/snap/microk8s/common/run/containerd.sock \
	./target/release/agent
```
> Note that its important to not run this as `sudo` and make sure Kube Config points to one with `admin` permissions (Krustlet bootstrap file does not work for this).
> Also note to apply the crds directory before running the Agent, as it uses them to connect with the Kubernetes node.

## Start the gRPC proxy

The rest of this tutorial will be done from this repository.

Now that the Agent is running, we can start the discovery process. Once we run the gRPC proxy it will communicate with the Agent, informing its ready to start finding devices for that specific protocol.
The gRPC proxy does not do any discovery, it is responsible for informing the Wasi Discovery Handlers about current constraints and passing to the Agent the list of discovered devices it receives.

```
cargo build -p dh-grpc-proxy --release
RUST_LOG=info \
    DISCOVERY_HANDLER_NAME=debugEcho \
    DISCOVERY_HANDLERS_DIRECTORY=~/device-plugins \
    AGENT_NODE_NAME=krustlet \
    ./target/release/dh-grpc-proxy
```
> Note that we are using the proxy to simulate a Debug Echo Discovery Handler but it is a universal program and support any future DHs.

## Apply Debug Echo Discovery Configuration

Now everything that should be running locally is already active, the rest of this demo will be focused on kubernetes components.
Now we need to inform the Agent we need to start looking for these devices by applying the Discovery Configuration. As soon as it’s applied the Agent will look for registered Discovery Handlers for this specific protocol (In our case, Debug Echo) and start the discovery process.

```
kubectl apply -f deployment/debug_echo_configuration.yaml
```
> Note that now new logs have appeared on the gRPC proxy as it already received the discovery details from the Agent.

## Deploy Wasi Debug Echo

Now the gRPC proxy should have successfully connected with the Akri's Agent and the input file was already written on the correct directory. The gRPC proxy is now waiting for the output file to be written by our WebAssembly application, we can deploy it now.

```
kubectl apply -f ./deployment/wasi_debug_echo.yaml 
```

## Checking the new resources

Now the Agent should have informed the Krustlet node about the new resources and are now advertised by the nodes. We can this out with the following command.

```
kubectl get akrii
```

## Requesting resources from Krustlet

Now that the devices are already been advertised as Kubernetes resources we can deploy a WebAssembly module that request them.

```
kubectl apply -f ./deployment/wasm_pod_using_wde.yaml
```

## Conclusion

During this demo we have showcased the usage of devices discovered by a Wasm Discovery Handler by a Wasm module inside Krustlet. This allows us to now use Akri in the WebAssembly environment and new Discovery Handlers and usages can build up from that.
