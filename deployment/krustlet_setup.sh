export KUBECONFIG=${HOME}/.krustlet/config/kubeconfig \
    # KRUSTLET_NODE_IP=172.17.0.1 \
    KRUSTLET_DEVICE_PLUGINS_DIR=${HOME}/device-plugins

chmod +x deployment/bootstrap.sh

./deployment/bootstrap.sh

echo "Krustlet is running :)"
krustlet-wasi \
  --node-ip=127.0.0.1 \
  --node-name=krustlet \
  --bootstrap-file=${HOME}/.krustlet/config/bootstrap.conf