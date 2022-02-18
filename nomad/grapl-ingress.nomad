locals {
  web_ui_port = 1234
}

job "grapl-ingress" {
  datacenters = ["dc1"]

  # meaning it runs on every agent
  type = "system"

  group "ingress-group" {
    # Expose grapl-web-ui, which is on the bridge network, to host's localhost.

    network {
      mode = "host"
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
              protocol = "tcp"
              service {
                # the upstream service
                name = "web-ui"
              }
            }
          }
        }
      }
    }
  }

  group "terminating-group" {
    # Expose AWS, be it real AWS or Localstack, as a Consul Connect service
    network {
      mode = "bridge"
    }

    service {
      name = "s3-gateway"

      connect {
        gateway {
          proxy {}
          terminating {
            service {
              name = "s3"
            }
          }
        }
      }
    }
  }
}