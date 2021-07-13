import unittest
import uuid

from grapl_common.envelope import pyuuid_to_proto_uuid, proto_uuid_to_pyuuid


class TestProtoUuidv4(unittest.TestCase):
    # A basic test to ensure that conversions between uuid representations preserve the uuid
    def test_uuid_roundtrip(self) -> None:
        original_uuid = uuid.uuid4()
        proto_uuid = pyuuid_to_proto_uuid(original_uuid)
        converted_original = proto_uuid_to_pyuuid(proto_uuid)
        converted_proto = pyuuid_to_proto_uuid(proto_uuid)
        self.assertEqual(original_uuid, converted_original)
        self.assertEqual(proto_uuid, converted_proto)
