from grapl_analyzerlib.schemas.schema_builder import (
    NodeSchema,
    OneToMany,
    ManyToOne,
    ManyToMany,
)


class ProcessSchema(NodeSchema):
    def __init__(self):
        super(ProcessSchema, self).__init__()
        (
            self.with_int_prop("process_id")
            .with_int_prop("created_timestamp")
            .with_str_prop("asset_id")
            .with_int_prop("terminate_time")
            .with_str_prop("image_name")
            .with_str_prop("process_name")
            .with_str_prop("arguments")
            .with_forward_edge("children", OneToMany(ProcessSchema), "parent")
            .with_forward_edge("bin_file", ManyToOne(FileSchema), "spawned_from")
            .with_forward_edge("created_files", OneToMany(FileSchema), "creator")
            .with_forward_edge("deleted_files", OneToMany(FileSchema), "deleter")
            .with_forward_edge("read_files", ManyToMany(FileSchema), "readers")
            .with_forward_edge("wrote_files", ManyToMany(FileSchema), "writers")
            .with_forward_edge(
                "created_connections",
                ManyToMany(ProcessOutboundConnectionSchema),
                "connections_from",
            )
            .with_forward_edge(
                "inbound_connections",
                ManyToMany(ProcessInboundConnectionSchema),
                "bound_by",
            )
            # .with_forward_edge('bound_connections', [uid])
        )

    @staticmethod
    def self_type() -> str:
        return "Process"


from grapl_analyzerlib.schemas.file_schema import FileSchema
from grapl_analyzerlib.schemas.process_inbound_network_connection_schema import (
    ProcessInboundConnectionSchema,
)
from grapl_analyzerlib.schemas.process_outbound_network_connection_schema import (
    ProcessOutboundConnectionSchema,
)
