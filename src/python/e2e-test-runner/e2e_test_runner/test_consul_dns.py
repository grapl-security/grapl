"""
Smoke test to ensure that consul dns is available from within the service mesh
"""
import socket

import pytest


# TODO re-enable this after the fix to Nomad DNS is released
@pytest.mark.skip(reason="Waiting on a Nomad release with the fix for this")
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
