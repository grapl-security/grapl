import { registerInstrumentations } from "@opentelemetry/instrumentation";
import { GraphQLInstrumentation } from "@opentelemetry/instrumentation-graphql";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-base";
import { NodeTracerProvider } from "@opentelemetry/sdk-trace-node";
import { HttpInstrumentation } from "@opentelemetry/instrumentation-http";
import { ExpressInstrumentation } from "@opentelemetry/instrumentation-express";
import { Resource } from "@opentelemetry/resources";
import { SemanticResourceAttributes } from "@opentelemetry/semantic-conventions";
import { ZipkinExporter } from "@opentelemetry/exporter-zipkin";

const provider: NodeTracerProvider = new NodeTracerProvider({
    resource: new Resource({
        [SemanticResourceAttributes.SERVICE_NAME]: "graphql-service",
    }),
});

provider.addSpanProcessor(
    new BatchSpanProcessor(
        new ZipkinExporter({
            serviceName: "graphql-endpoint",
        })
    )
);
provider.register();

registerInstrumentations({
    instrumentations: [
        new GraphQLInstrumentation(),
        new HttpInstrumentation(),
        new ExpressInstrumentation(),
    ],
});
