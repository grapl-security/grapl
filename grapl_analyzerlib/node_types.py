from typing import TypeVar

import grapl_analyzerlib.file_node as file_node

import grapl_analyzerlib.outbound_connection_node as outbound_connection_node
import grapl_analyzerlib.dynamic_node as dynamic_node
import grapl_analyzerlib.process_node as process_node
import grapl_analyzerlib.external_ip_node as external_ip_node
import grapl_analyzerlib.entities as entities


PV = TypeVar("PV", bound=process_node.ProcessView)
PQ = TypeVar("PQ", bound=process_node.ProcessQuery)

FV = TypeVar("FV", bound=file_node.FileView)
FQ = TypeVar("FQ", bound=file_node.FileQuery)

OCV = TypeVar("OCV", bound=outbound_connection_node.OutboundConnectionView)
OCQ = TypeVar("OCQ", bound=outbound_connection_node.OutboundConnectionQuery)

EIPV = TypeVar("EIPV", bound=external_ip_node.ExternalIpView)
EIPQ = TypeVar("EIPQ", bound=external_ip_node.ExternalIpQuery)

N = TypeVar("N", bound=entities.NodeView)
S = TypeVar("S", bound=entities.SubgraphView)

DNQ = TypeVar('DNQ', bound=dynamic_node.DynamicNodeQuery)
DNV = TypeVar('DNV', bound=dynamic_node.DynamicNodeView)

PluginNodeView = TypeVar('PluginNodeView')
