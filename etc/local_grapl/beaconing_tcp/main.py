# beaconing
import urllib.request


from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import *

def diff_timestamps(timestamps: [int]) -> [int]:
    diffs = []
    timestamp_iter = iter(timestamps)
    for timestamp_a in timestamp_iter:
        try:
            timestamp_b = next(timestamp_iter)
        except StopIteration:
            break
        diffs.append(timestamp_a - timestamp_b)
    return diffs

def is_beaconing(timestamps: [int]) -> bool:
    sorted_timestamps = list(timestamps)
    sorted_timestamps.sort(reverse=True)

    if len(timestamps) < 10:
        return None

    diffs = diff_timestamps(timestamps)

    _ks, d = ss.kstest(
        diffs,
        ss.randint.cdf,
        args=(sorted_timestamps[0], sorted_timestamps[-1])
    )
    if d != d:
        return d >= 0.5
    return None

class BeaconingTcp(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        return (
            ProcessQuery()
                .with_process_socket_outbound(
                    NetworkSocketAddressQuery()
                        .with_tcp_connection_to_a(
                            TcpConnectionQuery()
                        )
                )
        )

    def on_response(self, response_view: ProcessView, output: Any) -> None:
        # indexing into first element for now until we no longer need to workaround the 
        # issue with single edges
        timestamps = [netsock_address_src.get_tcp_connection_to_a()[0].get_created_timestamp() for c in view.process_socket_outbound]

        if is_beaconing(timestamps):
            output.send(
                            ExecutionHit(
                                analyzer_name="Process beaconing",
                                node_view=response_view,
                                risk_score=25,
                                lenses=[
                                    ("analyzer_name", "Process beaconing"),
                                ],
                                # Mark the dropper and its child processes as risky
                                risky_node_keys=[
                                    response_view.node_key
                                ],
                            )
                        )