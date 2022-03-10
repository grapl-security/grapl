from opentelemetry import trace
from opentelemetry.exporter.zipkin.json import ZipkinExporter
from opentelemetry.instrumentation.botocore import BotocoreInstrumentor
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor


def get_tracer(service_name: str, module_name: str) -> trace.Tracer:
    """
    This gets a tracer for instrumenting python apps manually with opentelemetry.

    Usage
    set OTEL_EXPORTER_ZIPKIN_ENDPOINT = "http://100.115.92.202:9411/api/v2/spans" as an environment variable

    from grapl_common.grapl_tracer import get_tracer
    ...
    TRACER = get_tracer(service_name='provisioner', module_name=__name__)
    ...
    foo():
        with TRACER.start_as_current_span("foo"):
            ...

    :param service_name: App/Service name ie provisioner
    :param module_name: This should be __name__
    :return: Tracer:
    """
    provider = TracerProvider(
        resource=Resource.create(
            {
                "service.name": service_name,
                # TODO add things like instance_id, etc here
            }
        )
    )
    # we're using the zipkin compatible endpoint in Jaegar for now.
    # Long-term we'll switch to OTEL
    zipkin_exporter = ZipkinExporter()
    processor = BatchSpanProcessor(zipkin_exporter)
    provider.add_span_processor(processor)
    trace.set_tracer_provider(provider)

    BotocoreInstrumentor().instrument()

    return trace.get_tracer(module_name)
