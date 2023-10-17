#!/usr/bin/env sh
set -eux -o pipefail
nerdctl build . -t expedition-backend:latest
nerdctl save expedition-backend:latest | sudo nerdctl --address /var/run/k3s/containerd/containerd.sock --namespace k8s.io load