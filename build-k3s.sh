#!/usr/bin/env sh
set -eux -o pipefail
nerdctl build . -t expedition-backend
nerdctl save expedition-backend | sudo nerdctl --address /var/run/k3s/containerd/containerd.sock --namespace k8s.io load