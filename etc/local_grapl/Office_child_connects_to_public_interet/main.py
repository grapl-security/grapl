from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import *

from grapl_analyzerlib.nodes.sysmon import ProcessView

import ipaddress

def process_context(process: ProcessView) -> None:
    process.get_pid()
    process.get_guid()
    process.get_exe()


def expand_trees(process: ProcessView):
    process.get_created_files()
    process.get_process_socket_outbound()
    process.get_process_exe()

    for child in (process.get_children() or []):
        expand_trees(child)

class OfficeChildConnectsToInternet(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        office_apps = ["word.exe"]

        #TODO: consider recursion: any (grand)child makes a connection to the internet.
        # attacker could avoid this, not just easily, but accidentally by running via cmd.exe
        # intermediate.
        return (
            ProcessQuery()
                .with_exe(eq=office_apps)
                .with_process_socket_outbound()
                .with_children(
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
        )

    def on_response(self, dropper: ProcessView, output: Any) -> None:
        for netsock_address_src in view.process_socket_outbound:
            for tcp_connection_a in netsock_address_src.get_tcp_connection_to_a():
                for netsock_address_dst in tcp_connection_a.get_tcp_connection_to_b():
                    for ipv4_address in netsock_address_dst.get_socket_ipv4_address():
                        if ipaddress.ip_address(ipv4_address.get_address()).is_global:
                            output.send(
                                ExecutionHit(
                                    analyzer_name="Office child process connects to internet",
                                    node_view=response_view,
                                    risk_score=25,
                                    lenses=[
                                        ("analyzer_name", "Office child process connects to internet"),
                                    ],
                                    risky_node_keys=[
                                        response_view.node_key
                                    ],
                                )
                            )
