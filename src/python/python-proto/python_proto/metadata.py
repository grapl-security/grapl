# How users should feed metadata into a wrapped grpc client
GrpcOutboundMetadata = dict[str, str | bytes]

# How the grpc client actually has to send data over the wire
# N tuples of (str, str)
GrpcOutboundMetadataRaw = tuple[tuple[str, str | bytes], ...]


def metadata_to_raw(input: GrpcOutboundMetadata | None) -> GrpcOutboundMetadataRaw:
    if not input:
        return tuple()
    return tuple(input.items())
