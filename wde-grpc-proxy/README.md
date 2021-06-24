# Akri Discovery Handler Template
A template for accelerating creating a Discovery Handler for Akri in Rust. 

## About
A Discovery Handler is anything that implements Akri's the `Discover` service and `Registration` client defined in
Akri's [discovery gRPC interface](https://github.com/deislabs/akri/blob/main/discovery-utils/proto/discovery.proto).
This can be done in any language using Akri's proto file. This template creates a `DiscoveryHandler` that implements the
`Discover` service and registers it with the Akri Agent.

## Usage
## Use `cargo generate` to clone this template
This template is pulled via the [`cargo-generate`](https://github.com/cargo-generate/cargo-generate) developer tool.
1. Install `cargo-generate` `cargo install cargo-generate`
2. Pull down this template `cargo generate --git https://github.com/kate-goldenring/akri-discovery-handler-template.git`
3. Fill in the [`discover`](src/discovery_handler.rs) logic of your Discovery Handler
4. Build the Discovery Handler and push it to your container registry (assumed GHCR below):
    ```sh
    HOST="ghcr.io"
    USER=[[GITHUB-USER]]
    DH="discovery-handler"
    TAGS="v1"

    DH_IMAGE="${HOST}/${USER}/${DH}"
    DH_IMAGE_TAGGED="${DH_IMAGE}:${TAGS}"

    docker build \
    --tag=${DH_IMAGE_TAGGED} \
    --file=./Dockerfile.discovery-handler \
    . && \
    docker push ${DH_IMAGE_TAGGED}
    ```
5. Deploy Akri with your custom Discovery Handler
    ```sh
    helm repo add akri-helm-charts https://deislabs.github.io/akri/
    helm install akri akri-helm-charts/akri-dev \
    --set imagePullSecrets[0].name="crPullSecret" \
    --set custom.discovery.enabled=true  \
    --set custom.discovery.image.repository=$DH_IMAGE \
    --set custom.discovery.image.tag=$TAGS \
    --set custom.discovery.name=akri-<name>-discovery  \
    --set custom.configuration.enabled=true  \
    --set custom.configuration.name=akri-<name>  \
    --set custom.configuration.discoveryHandlerName=<name> \
    --set custom.configuration.discoveryDetails=<filtering info>
    ```