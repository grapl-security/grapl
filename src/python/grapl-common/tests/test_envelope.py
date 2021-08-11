import uuid

from grapl_common.envelope import Uuid

# FIXME: test the other classes, use hypothesis, actually test the contracts


def test_uuid_roundtrip() -> None:
    original_uuid: uuid.UUID = uuid.uuid4()
    proto_uuid = Uuid.from_uuid(uuid_=original_uuid)._into_proto()
    converted_original = Uuid._from_proto(proto_uuid=proto_uuid).into_uuid()
    converted_proto = Uuid.from_uuid(uuid_=converted_original)._into_proto()
    assert original_uuid == converted_original
    assert proto_uuid == converted_proto
