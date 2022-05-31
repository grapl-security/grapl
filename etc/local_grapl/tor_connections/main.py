import urllib.request
from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import *

from grapl_analyzerlib.nodes.sysmon import ProcessView

# Latest ndoe list is available from: https://check.torproject.org/torbulkexitlist
#
# Instead of hitting the check.torproject.org for each execution, we could possibly:
#   - use caching service so we don't peg https://check.torproject.org/torbulkexitlist
#   - external service to push new versions of this analyzer with an updated static list.
def fetch_exit_nodes() -> [str]:
    contents = urllib.request.urlopen("https://check.torproject.org/torbulkexitlist").read()
    nodes = contents.splitlines()


def is_tor_exit_node(ip: str) -> bool:
    nodes = fetch_exit_nodes()
    if nodes:
        ip in nodes
    else:
        False

class TorConnections(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        return (
            ProcessQuery()
                .with_process_socket_outbound(
                    NetworkSocketAddressQuery()
                        .with_tcp_connection_to_a(
                            TcpConnectionQuery()
                                .with_tcp_connection_to_b(
                                    NetworkSocketAddressQuery()
                                        .with_socket_ipv4_address(
                                            IpV4AddressQuery()
                                                .with_address()
                                        )
                                )
                        )
                )
        )

    def on_response(self, response_view: ProcessView, output: Any) -> None:
        # something something don't match on something you've matched before
        # colin had a thing for this where is it
        #
        # TODAY we need to make sure we only add the node_key for the process
        # in the risky_node_keys. just make sure to do all the get_ shit to expand.
        #
        # alternatively we could send multiple hits, one per process + connection pair,
        # where we send for each, loop through
        #
        # prefer: loop through each connection, send a hit for each pair of process + connection pair,
        # where the only different between them all is the NetSocketAddress node key

        # we currently have a hack using ToMany everywhere to work around some issue with edges.
        # hmmm this is mixing both use of identity-only fields and avoiding them, revisit.
        for netsock_address_src in view.process_socket_outbound:
        for tcp_connection_a in netsock_address_src.get_tcp_connection_to_a():
            for netsock_address_dst in tcp_connection_a.get_tcp_connection_to_b():
                for ipv4_address in netsock_address_dst.get_socket_ipv4_address():
                    if is_tor_exit_node(ipv4_address.get_address()):
                        output.send(
                            ExecutionHit(
                                analyzer_name="TorConnections",
                                node_view=response_view,
                                risk_score=25,
                                lenses=[
                                    ("analyzer_name", "TorConnections"),
                                ],
                                # Mark the dropper and its child processes as risky
                                risky_node_keys=[
                                    response_view.node_key
                                ],
                            )
                        )