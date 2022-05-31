"""
Smoke test to ensure that consul dns is available from within the service mesh
"""
import socket

import pytest


# TODO re-enable this after the fix to Nomad DNS is released. The fix twunderlich/skip-consul-dns-test-for-now has been
#  merged to main. Hopefully it will be released as Nomad 1.3.2
@pytest.mark.skip(reason="Waiting on a Nomad release (1.3.2?) with the fix for this")
def test_if_consul_dns_resolves() -> None:
    try:
        ipaddress = socket.gethostbyname("web-ui.service.consul")
    except socket.gaierror:
        ipaddress = ""
    assert ipaddress


def test_if_external_dns_resolves() -> None:
    try:
        ipaddress = socket.gethostbyname("google.com")
    except socket.gaierror:
        ipaddress = ""
    assert ipaddress
