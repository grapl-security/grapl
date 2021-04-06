FROM grapl-js-base AS graphql-endpoint-build
RUN mkdir -p lambda
# Install the dependencies separately to leverage Docker cache
WORKDIR /home/grapl/lambda
COPY --chown=grapl js/graphql_endpoint/tsconfig.json tsconfig.json
COPY --chown=grapl js/graphql_endpoint/package.json package.json
COPY --chown=grapl js/graphql_endpoint/package-lock.json package-lock.json
RUN npm install
RUN rm -rf node_modules/grpc/build/
# Copy graphql sources
COPY --chown=grapl js/graphql_endpoint/modules modules
COPY --chown=grapl js/graphql_endpoint/server.ts server.ts
# This compiles the typescript and spits out the built js files to `lambda/ts_compiled'
RUN npx tsc
RUN cp -r ./node_modules ./ts_compiled/node_modules
WORKDIR /home/grapl

# zip
FROM graphql-endpoint-build AS graphql-endpoint-zip
RUN test -d lambda/ts_compiled/node_modules && echo "Okay, found node_modules"
RUN cd lambda/ts_compiled && \
    zip -q9r /home/grapl/lambda.zip ./

# deploy... this is only used in Local Grapl at the moment
FROM grapl-js-base AS graphql-endpoint
COPY --chown=grapl --from=graphql-endpoint-build /home/grapl/lambda lambda
WORKDIR /home/grapl/lambda/ts_compiled
COPY --chown=grapl js/graphql_endpoint/package.json package.json
COPY --chown=grapl js/graphql_endpoint/package-lock.json package-lock.json
COPY --chown=grapl js/graphql_endpoint/start_potentially_with_debugger.sh \
  start_potentially_with_debugger.sh
CMD yarn start server
