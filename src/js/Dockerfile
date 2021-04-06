FROM grapl-build-base AS grapl-js-base
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
RUN nvm install --lts
RUN npm install -g yarn

FROM grapl-js-base AS grapl-js-build
RUN
