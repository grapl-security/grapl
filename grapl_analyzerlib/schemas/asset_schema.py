from grapl_analyzerlib.schemas.schema_builder import NodeSchema, OneToMany


class AssetSchema(NodeSchema):
    def __init__(self) -> None:
        super(AssetSchema, self).__init__()
        (
            self
            .with_str_prop("hostname")
            .with_forward_edge(
                "asset_ip",
                # An process_asset can have multiple IP address
                OneToMany(IpAddressSchema),
                "ip_assigned_to",
            )
            .with_forward_edge(
                'asset_processes',
                OneToMany(ProcessSchema),
                'process_asset'
            )
            .with_forward_edge(
                'files_on_asset',
                OneToMany(FileSchema),
                'file_asset'
            )
        )

    @staticmethod
    def self_type() -> str:
        return "Asset"


from grapl_analyzerlib.schemas.ip_address_schema import IpAddressSchema
from grapl_analyzerlib.schemas.process_schema import ProcessSchema
from grapl_analyzerlib.schemas.file_schema import FileSchema