import { registerInstrumentations } from '@opentelemetry/instrumentation';
import { GraphQLInstrumentation } from '@opentelemetry/instrumentation-graphql';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-base';
import { NodeTracerProvider } from '@opentelemetry/sdk-trace-node';
import { HttpInstrumentation } from '@opentelemetry/instrumentation-http';
import { ExpressInstrumentation } from '@opentelemetry/instrumentation-express';
import { Resource } from '@opentelemetry/resources';
import { SemanticResourceAttributes }  from '@opentelemetry/semantic-conventions';
import { ZipkinExporter } from "@opentelemetry/exporter-zipkin";

const provider: NodeTracerProvider = new NodeTracerProvider({
  resource: new Resource({
    [SemanticResourceAttributes.SERVICE_NAME]: 'graphql-service',
  }),
});

provider.addSpanProcessor(new BatchSpanProcessor(new ZipkinExporter({url: "http://100.115.92.202:9411/api/v2/spans", serviceName: "graphql-endpoint"})));
provider.register();

registerInstrumentations({
  instrumentations: [
    new GraphQLInstrumentation(
//     {
// //       allowAttributes: true,
// //       depth: 2,
// //       mergeItems: true,
//     }
    ),
    new HttpInstrumentation(),
    new ExpressInstrumentation(),
  ],
});