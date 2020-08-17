from grapl_analyzerlib.nodes.process import ProcessSchema
from grapl_analyzerlib.nodes.asset import AssetSchema
from grapl_analyzerlib.nodes.file import FileSchema
from grapl_analyzerlib.nodes.ip_address import IpAddressSchema
from grapl_analyzerlib.nodes.ip_connection import IpConnectionSchema
from grapl_analyzerlib.nodes.ip_port import IpPortSchema
from grapl_analyzerlib.nodes.lens import LensSchema
from grapl_analyzerlib.nodes.process_inbound_connection import (
    ProcessInboundConnectionSchema,
)
from grapl_analyzerlib.nodes.process_outbound_connection import (
    ProcessOutboundConnectionSchema,
)
from grapl_analyzerlib.nodes.risk import RiskSchema

AssetSchema().init_reverse()
FileSchema().init_reverse()
IpAddressSchema().init_reverse()
IpPortSchema().init_reverse()
IpConnectionSchema().init_reverse()
LensSchema().init_reverse()
ProcessInboundConnectionSchema().init_reverse()
ProcessOutboundConnectionSchema().init_reverse()
RiskSchema().init_reverse()
ProcessSchema().init_reverse()
