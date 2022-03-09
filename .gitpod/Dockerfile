FROM gitpod/workspace-full:commit-8fc141dbdd92030a435ead06617c6d37651d8312

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ARG PACKER_VERSION=1.7.8
ARG VAULT_VERSION=1.9.1
ARG NOMAD_VERSION=1.2.2
ARG CONSUL_VERSION=1.10.4
ARG PULUMI_VERSION=3.18.1
ARG CLOUDSMITH_VERSION=0.30.1
ARG JMESPATH_VERSION=0.10.0
ARG AWS_SSO_UTIL_VERSION=4.25.0
ARG AWSCLI_VERSION=2.1.39

# Don't care about using sudo here
# hadolint ignore=DL3004
RUN curl --fail \
         --silent \
         --show-error \
         --location https://apt.releases.hashicorp.com/gpg | sudo apt-key add - \
    && sudo apt-add-repository \
            "deb [arch=amd64] https://apt.releases.hashicorp.com $(lsb_release --codename --short) main" \
    && sudo apt-get update \
    && sudo apt-get install --yes --no-install-recommends \
            packer=${PACKER_VERSION} \
            vault=${VAULT_VERSION} \
            nomad=${NOMAD_VERSION} \
            consul=${CONSUL_VERSION} \
    && sudo rm -rf /var/lib/apt/lists/* \
    && sudo mkdir --parents /opt/grapl/bin \
    && curl --fail\
            --silent \
            --show-error \
            --location \
            --output pulumi.tar.gz \
            https://get.pulumi.com/releases/sdk/pulumi-v${PULUMI_VERSION}-linux-x64.tar.gz \
    && sudo tar --extract \
            --gunzip \
            --verbose \
            --file=pulumi.tar.gz \
            --strip-components=1 \
            --directory=/opt/grapl/bin \
    && rm pulumi.tar.gz

RUN curl --proto "=https" \
    --tlsv1.2 \
    --location \
    --verbose \
    --output awscliv2.zip \
    "https://awscli.amazonaws.com/awscli-exe-linux-x86_64-${AWSCLI_VERSION}.zip" \
    && unzip awscliv2.zip \
    && sudo ./aws/install \
    --bin-dir /opt/grapl/bin \
    --install-dir /opt/grapl/aws-cli \
    && rm -Rf aws awscliv2.zip
ENV AWS_CLI_AUTO_PROMPT=on

RUN sudo mkdir -p /opt/grapl/pipx \
    && sudo chown gitpod:gitpod /opt/grapl/pipx \
    && sudo chown gitpod:gitpod /opt/grapl/bin

USER gitpod

ENV PATH=/opt/grapl/bin:${PATH}

ENV PIPX_BIN_DIR=/opt/grapl/bin
ENV PIPX_HOME=/opt/grapl/pipx

RUN pip install pipx
RUN pipx install cloudsmith-cli==${CLOUDSMITH_VERSION}
RUN pipx install jmespath==${JMESPATH_VERSION}
RUN pipx install aws-sso-util==${AWS_SSO_UTIL_VERSION}

RUN npm install --global bash-language-server@2.0.0

COPY .bashrc.d/* /home/gitpod/.bashrc.d/
COPY setup-grapl-sso.sh /opt/grapl/bin/
