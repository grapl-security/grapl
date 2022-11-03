FROM otel/opentelemetry-collector-contrib:0.63.0 as otel-collector

# We are rebuilding the otel containers so we can add a writable directory (in this case /tmp)
FROM gcr.io/distroless/static-debian11

# Pull files from the otel-collector
COPY --from=otel-collector /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=otel-collector --chmod=755 /otelcol-contrib /otelcol-contrib
COPY --from=otel-collector /etc/otelcol-contrib/config.yaml /etc/otelcol-contrib/config.yaml

ENTRYPOINT ["/otelcol-contrib"]
CMD ["--config", "/etc/otelcol-contrib/config.yaml"]
EXPOSE 4317 55678 55679