from opentelemetry import trace
from opentelemetry.exporter.zipkin.json import ZipkinExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import Tracer, TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor


def get_tracer(service_name: str, module_name: str) -> Tracer:
    """
    This gets a tracer for instrumenting python apps manually with opentelemetry.

    Usage
    from grapl_common.grapl_tracer import get_tracer
    ...
    TRACER = get_tracer(service_name='provisioner', module_name=__name__)
    ...
    foo():




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
    zipkin_exporter = ZipkinExporter(
        # version=Protocol.V2
        # optional:
        endpoint="http://100.115.92.202:9411/api/v2/spans",
        # local_node_ipv4="192.168.0.1",
        # local_node_ipv6="2001:db8::c001",
        # local_node_port=31313,
        # max_tag_value_length=256
        # timeout=5 (in seconds)
    )
    processor = BatchSpanProcessor(zipkin_exporter)
    provider.add_span_processor(processor)
    trace.set_tracer_provider(provider)
    return trace.get_tracer(module_name)
