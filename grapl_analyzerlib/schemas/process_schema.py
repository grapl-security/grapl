from grapl_analyzerlib.schemas.schema_builder import NodeSchema


class ProcessSchema(NodeSchema):
    def __init__(self):
        super(ProcessSchema, self).__init__()
        (
            self
                .with_int_prop('process_id')
                .with_int_prop('created_timestamp')
                .with_str_prop('asset_id')
                .with_int_prop('terminate_time')
                .with_str_prop('image_name')
                .with_str_prop('process_name')
                .with_str_prop('arguments')
                .with_forward_edge('children', [ProcessSchema])
                .with_forward_edge('bin_file', FileSchema)
                .with_forward_edge('created_files', [FileSchema])
                .with_forward_edge('deleted_files', [FileSchema])
                .with_forward_edge('read_files', [FileSchema])
                .with_forward_edge('wrote_files', [FileSchema])
                .with_forward_edge('created_connections', [OutboundConnectionSchema])
            # .with_forward_edge('bound_connections', [uid])
        )

    @staticmethod
    def self_type() -> str:
        return "Process"


from grapl_analyzerlib.schemas.file_schema import FileSchema
from grapl_analyzerlib.schemas.outbound_connection_schema import OutboundConnectionSchema
