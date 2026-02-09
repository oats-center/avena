FROM quay.io/fedora/fedora-bootc:43

ARG BUILD_PROFILE=debug

RUN dnf install -y \
    podman \
    iproute \
    iputils \
    procps-ng \
    vim-minimal && \
    dnf clean all

RUN mkdir -p /nats/cfg /var/lib/avena /etc/containers/systemd/nats

COPY target/${BUILD_PROFILE}/avenad /usr/local/bin/avenad
COPY target/${BUILD_PROFILE}/avenactl /usr/local/bin/avenactl
COPY scripts/avenad.service /etc/systemd/system/avenad.service

RUN chmod +x /usr/local/bin/avenad /usr/local/bin/avenactl && \
    systemctl enable avenad.service
