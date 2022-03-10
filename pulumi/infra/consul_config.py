import json
from typing import Optional

import pulumi_consul as consul

import pulumi


class ConsulConfig(pulumi.ComponentResource):
    """
    Consul config entries
    """

    def __init__(
        self,
        name: str,
        tracing_endpoint: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ConsulConfig", name, None, opts)

        # Instead of using a reading a hcl file or a template, we're just going to define it here as a plain python dict
        # so its easy to pass in a url.
        config = {
            "Config": [
                {
                    "envoy_extra_static_clusters_json": json.dumps(
                        {
                            "name": "zipkin",
                            "type": "STRICT_DNS",
                            "connect_timeout": "5s",
                            "load_assignment": {
                                "cluster_name": "zipkin",
                                "endpoints": [
                                    {
                                        "lb_endpoints": [
                                            {
                                                "endpoint": {
                                                    "address": {
                                                        "socket_address": {
                                                            "address": tracing_endpoint,
                                                            "port_value": 9411,
                                                        }
                                                    }
                                                }
                                            }
                                        ]
                                    }
                                ],
                            },
                        }
                    ),
                    "envoy_stats_flush_interval": "10s",
                    "envoy_tracing_json": json.dumps(
                        {
                            "http": {
                                "name": "envoy.tracers.zipkin",
                                "typedConfig": {
                                    "@type": "type.googleapis.com/envoy.config.trace.v3.ZipkinConfig",
                                    "collector_cluster": "zipkin",
                                    "collector_endpoint_version": "HTTP_JSON",
                                    "collector_endpoint": "/api/v2/spans",
                                    "shared_span_context": False,
                                    "trace_id_128bit": True,
                                },
                            }
                        }
                    ),
                    "prometheus_bind_addr": "0.0.0.0:9102",
                    "protocol": "grpc",
                }
            ]
        }

        consul.ConfigEntry(
            resource_name=f"{name}-proxy-defaults",
            kind="proxy-defaults",
            name="global",
            config_json=json.dumps(config),
            opts=pulumi.ResourceOptions.merge(
                opts, pulumi.ResourceOptions(parent=self)
            ),
        )
