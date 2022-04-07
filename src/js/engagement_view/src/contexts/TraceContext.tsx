import React from "react";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-base";
import { WebTracerProvider } from "@opentelemetry/sdk-trace-web";
import { BaseOpenTelemetryComponent } from "@opentelemetry/plugin-react-load";
import { ZipkinExporter } from "@opentelemetry/exporter-zipkin";
import { diag, DiagConsoleLogger } from "@opentelemetry/api";
import { Resource } from "@opentelemetry/resources";
import { SemanticResourceAttributes } from "@opentelemetry/semantic-conventions";
import { registerInstrumentations } from "@opentelemetry/instrumentation";
import { getWebAutoInstrumentations } from "@opentelemetry/auto-instrumentations-web";
import { ZoneContextManager } from "@opentelemetry/context-zone";

const serviceName = "frontend-ui";

const provider = new WebTracerProvider({
    resource: new Resource({
        [SemanticResourceAttributes.SERVICE_NAME]: serviceName,
    }),
});

const exporter = new ZipkinExporter({
    url: "http://localhost:9411/api/v2/spans",
    // for whatever reason the Content-Type is being set incorrectly and causing spans to be rejected.
    headers: {
        "Content-Type": "application/json",
    },
});

provider.addSpanProcessor(new BatchSpanProcessor(exporter));

provider.register({
    // Changing default contextManager to use ZoneContextManager - supports asynchronous operations - optional
    contextManager: new ZoneContextManager(),
});

// Registering instrumentations
registerInstrumentations({
    instrumentations: [
        getWebAutoInstrumentations({
            // load custom configuration for xml-http-request instrumentation
            "@opentelemetry/instrumentation-xml-http-request": {
                propagateTraceHeaderCorsUrls: [/.+/g],
            },
            // load custom configuration for fetch instrumentation
            "@opentelemetry/instrumentation-fetch": {
                propagateTraceHeaderCorsUrls: [/.+/g],
            },
        }),
    ],
});

BaseOpenTelemetryComponent.setTracer(serviceName);
diag.setLogger(new DiagConsoleLogger());

export type TraceProviderProps = {
    children?: React.ReactNode;
};

export default function TraceProvider({
    children,
}: TraceProviderProps): React.ReactElement {
    return <>{children}</>;
}
