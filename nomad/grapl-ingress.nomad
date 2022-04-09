variable "dns_server" {
  type        = string
  description = "The network.dns.server value. This should be equivalent to the host's ip in order to communicate with dnsmasq and allow consul dns to be available from within containers. This can be replaced as of Nomad 1.3.0 with variable interpolation per https://github.com/hashicorp/nomad/issues/11851."
}

locals {
  web_ui_port = 1234
  dns_servers = [var.dns_server]
}

job "grapl-ingress" {
  datacenters = ["dc1"]

  # meaning it runs on every agent
  type = "system"

  group "ingress-group" {
    # Expose grapl-web-ui, which is on the bridge network, to host's localhost.

    network {
      mode = "host"
      dns {
        servers = local.dns_servers
      }
    }

    service {
      name = "ingress-service"
      port = local.web_ui_port

      connect {
        gateway {
          # Consul Ingress Gateway Configuration Entry.
          ingress {
            listener {
              port     = local.web_ui_port
              protocol = "http"
              service {
                # the upstream service
                name  = "web-ui"
                hosts = ["*"]
              }
            }
          }
        }
      }
    }
  }
}