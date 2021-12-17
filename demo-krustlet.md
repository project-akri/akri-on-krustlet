# Krustlet using Akri devices demo
This is a demo to showcase  WebAssembly application running on Krustlet using fake "foo" devices discovered by Akri.

In the demo, a local running Akri Agent will communicate with Krustlet device plugin manager and allow the creation of the resources discovered by the Wasm Discovery Handler. The result will be the architecture below, where local, Wasm, and containerized components are colored as grey, orange, and purple, respectively.

<img src="./KrustletUsingAkriDevicesDesign.png" alt="Krustlet integration architecture" style="padding-bottom: 10px padding-top: 10px;
margin-right: auto; display: block; margin-left: auto;"/>

## Start your kubernetes cluster

For this demo we are using Kind, but feel free to use any of your choice, Krustlet has documentation for most of them [here](https://docs.krustlet.dev/howto/).

## Start your Krustlet node

Krustlet has an automatic bootstrap process that gives the node the right authorizations to execute, you can find a tutorial for it [here](https://docs.krustlet.dev/howto/bootstrapping/).
For this demo we are using an unreleased version of Krustlet that enable the device plugin features. Because of that, we are running Krustlet directly from the main branch from GitHub using the command:

```
git clone https://github.com/krustlet/krustlet.git
cd krustlet
KRUSTLET_NODE_IP=172.17.0.1 \
	KRUSTLET_HOSTNAME=krustlet-wasi \
	KRUSTLET_NODE_NAME=krustlet-wasi \
	KRUSTLET_BOOTSTRAP_FILE=~/.krustlet/config/bootstrap.conf \
	just run
```
> Make sure the Node IP informed is present on the known hosts list (Manually add it if not present).

In another terminal, validate the Krustlet node's CSR and wait for the node to become ready.
```
kubectl certificate approve krustlet-wasi-tls
watch kubectl get no
```
You should see two nodes listed: one control-plane node and one krustlet-wasi node.

## Install Akri Controller and CRDs

Now, we are ready to install Akri using its Helm chart. Note that we are disabling installing the Agent, since it will be run locally on the Krustlet node. We are also not specifying any Discovery Handlers as we apply our Wasm one later. This command will install the Akri Controller and Akri's CRDs (Akri Configuration CRD and Akri Instance CRD).

```
helm repo add akri-helm-charts https://deislabs.github.io/akri/
helm install akri akri-helm-charts/akri \
 --set agent.enabled=false
```

## Clone and update Akri repository

Now, let's clone the Akri project and do some small changes to make it compatible with the Krustlet environment.

```
git clone https://github.com/deislabs/akri.git
cd akri
```

The Akri Agent's Device Plugin and Kubelet directories can configured as a `hostPath` when installing with Helm. Since we are running locally, we need to update `DEVICE_PLUGIN_PATH` and `KUBELET_SOCKET` constants in `/agent/src/util/constants.rs#L13` to represent the Krustlet directory `./.krustlet/device_plugins`. Now build the Agent.

```
cargo build -p agent --release
```
> Note: remove `--release` and specify the `./akri/target/debug/agent` target in the next step for faster build times.

### Run Akri Agent

Now, we are ready to run the Agent locally. The Agent informs the Krustlet node about new resources discovered by Discovery Handlers.

```
cd ~
RUST_LOG=info RUST_BACKTRACE=1 KUBECONFIG=~/.kube/config \
	DISCOVERY_HANDLERS_DIRECTORY=~/akri \
	AGENT_NODE_NAME=krustlet-wasi \
	HOST_CRICTL_PATH=/usr/local/bin/crictl \
	HOST_RUNTIME_ENDPOINT=/usr/local/bin/containerd \
	HOST_IMAGE_ENDPOINT=/usr/local/bin/containerd \
	./akri/target/release/agent
```
> Note: remove `--release` and change to `./target/debug/` agent for faster build times

> Note that it’s important to not run this as `sudo` and make sure Kube Config points to one with `admin` permissions (Krustlet bootstrap file does not work for this).

## Start the gRPC proxy

The rest of this tutorial will be done from this repository.

Now that the Agent is running, we can start the discovery process. Once we run the gRPC proxy it will communicate with the Agent, informing its ready to start finding devices for that specific protocol.
The gRPC proxy does not do any discovery, it is responsible for informing the Wasm Discovery Handlers about current constraints and passing to the Agent the list of discovered devices it receives.

```
mkdir /tmp/wde-dir
cargo build -p dh-grpc-proxy --release
RUST_LOG=info \
    DISCOVERY_HANDLER_NAME=debugEcho \
    DISCOVERY_HANDLERS_DIRECTORY=~/akri \
    AGENT_NODE_NAME=krustlet-wasi \
    ./target/release/dh-grpc-proxy
```
> We are creating the `/tmp/wde-dir` directory as it will be used for the communication between this gRPC proxy and the Discovery Handler.

> Note that we are using the proxy to simulate a Debug Echo Discovery Handler, but it is a universal program and support any future DHs.

## Deploy the debugEcho Configuraton 
To tell the Agent to start discovery, we need to deploy a Configuration. In the [Configuration for this demo](./deployment/debug-echo-configuration.yaml), we are specifying that we want to use the `debugEcho` Discovery Handler to discover on `foo0` device. We are also requesting that a Wasm module is deployed to discovered devices by setting the OCI image for the Wasm module in the `brokerPodSpec` section of the Configuration. The Node Selector `kubernetes.io/arch: "wasm32-wasi"` ensures that when the Akri Controller deploys it, it is run on the Krustlet node. 

Apply the Configuration.
```
kubectl apply -f ~/akri-on-krustlet/deployment/debug-echo-configuration.yaml
```
> Note: we can specify the discovery of more foo devices by adding more entries to the `descriptions` array. For example: 
> ```yaml
>     discoveryDetails: |+
>       descriptions:
>       - foo0
>       - foo1
>       - fooN
>

## Deploy Wasi Debug Echo

Now the gRPC proxy should have successfully connected with the Akri Agent and the input file was already written on the correct directory. The gRPC proxy is now waiting for discovered devices to be written to the output file by our WebAssembly discovery handler. We can deploy it now.

```
kubectl apply -f ~/akri-on-krustlet/deployment/wasi_debug_echo.yaml
```

## Checking the new resources

Now the Agent should have informed the Krustlet node about the new resource `foo0` and created an Akri Instance to represent it. The Akri Controller will see this Instance and automatically deploy our Wasm application to use it. Watch for the creation of the Instance and broker Pod.

```
watch kubectl get akrii,pods -o wide
```

Inspect the broker Wasm Pod and see that it is using one of the `foo` devices as a resource. Be sure to change the following command to match the ID of your Wasm pod:
```
kubectl describe pod krustlet-wasi-akri-debug-echo-<ID>-pod
```

The output should contain the following, with the device ID varying:
```
Containers:
  im-using-debug-echo-devices:
    Container ID:
    Image:          ghcr.io/rodz/wasi-debug-echo-hello-world:v1
    Image ID:
    Port:           <none>
    Host Port:      <none>
    State:          Running
      Started:      Wed, 08 Sep 2021 16:32:36 +0000
    Ready:          True
    Restart Count:  0
    Limits:
      akri.sh/akri-debug-echo-aae70c:  1
    Requests:
      akri.sh/akri-debug-echo-aae70c:  1
```

Now, look at the logs of the container, as they are outputting the name of the device, which was set in an environment variable and injected into the container due to the resource being requested:

```
kubectl logs krustlet-wasi-akri-debug-echo-aae70c-pod
```
The output should be:
```
Pod is running and using debugEcho device named: "foo0"
```

## Conclusion

During this demo we have showcased the usage of devices discovered by a Wasm Discovery Handler running on a Krustlet node. This allows us to now use Akri in the WebAssembly environment. Developments in WASI will enable us to discover real devices and compile Akri's other Discovery Handlers to Wasm without using a gRPC proxy.
