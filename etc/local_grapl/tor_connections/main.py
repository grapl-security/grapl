from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import *

from grapl_analyzerlib.nodes.sysmon import ProcessView

# def get_tor_exit_nodes() -> [String]
    # latest available from: https://check.torproject.org/torbulkexitlist
    #
    # wondering how we could/should update this periodically in Grapl
    #   opt 1: reach out to https://check.torproject.org/torbulkexitlist directly
    #   opt 2: use caching service so we don't peg https://check.torproject.org/torbulkexitlist
    #   opt 3: external service to push new versions of this analyzer with an updated list baked in.

# def is_tor_exit_node(ip: String) -> bool:


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
                                        .with_dst_ip_address()
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
        for netsock_address_src in response_view.process_socket_outbound:
            for tcp_connection_a in netsock_address.tcp_connection_to_a:
                for netsock_address_dst in tcp_connection_a.tcp_connection_to_b:
                    if is_tor_exit_node(netsock_address_dst.dst_ip_address):
                        output.send(
                            ExecutionHit(
                                analyzer_name="TorConnections",
                                node_view=response_view,
                                risk_score=75,
                                lenses=[
                                    ("analyzer_name", "TorConnections"),
                                ],
                                # Mark the dropper and its child processes as risky
                                risky_node_keys=[
                                    response_view.node_key
                                ],
                            )
                        )