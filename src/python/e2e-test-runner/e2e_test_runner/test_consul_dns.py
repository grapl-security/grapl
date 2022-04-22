"""
Smoke test to ensure that consul dns is available from within the service mesh
"""
import socket


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
