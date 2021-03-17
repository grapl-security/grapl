from typing import Optional

from infra import util

import pulumi


class EventEmitter(pulumi.ComponentResource):
    """
    Buckets that send events to SNS topics
    """

    def __init__(
        self, event_name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:

        super().__init__("grapl:EventEmitter", event_name, None, opts)

        logical_bucket_name = f"{event_name}-bucket"
        self.bucket = util.grapl_bucket(logical_bucket_name, sse=True, parent=self)

        self.register_outputs({})
