locals {
  web_ui_port = 1234
  # Use Consul agent's dns on port 8600
  dns_servers = ["${attr.unique.network.ip-address}:8600"]
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