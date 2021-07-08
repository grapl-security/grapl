from __future__ import annotations
from typing import (
    Dict,
    Iterator,
    MutableMapping,
)

from graplinc.grapl.api.services.v1beta1.types_pb2 import Meta, Envelope as _Envelope


class Envelope(object):
    def __init__(
            self, envelope: _Envelope
    ) -> None:
        self.envelope = envelope

    @staticmethod
    def from_proto(s: bytes) -> Envelope:
        envelope = _Envelope()
        envelope.ParseFromString(s)

        return Envelope(envelope)

    @property
    def metadata(self) -> Meta:
        return self.envelope.metadata

    @property
    def inner_type(self) -> str:
        return self.envelope.inner_type

    @property
    def inner_message(self) -> bytes:
        return self.envelope.inner_message

from grapl_analyzerlib.prelude import BaseView
