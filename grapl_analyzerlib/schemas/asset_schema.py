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
                'processes_on_asset',
                OneToMany(ProcessSchema),
                'process_asset'
            )
            .with_forward_edge(
                'files_on_asset',
                OneToMany(ProcessSchema),
                'file_asset'
            )
        )

    @staticmethod
    def self_type() -> str:
        return "Asset"


from grapl_analyzerlib.schemas import IpAddressSchema, ProcessSchema
