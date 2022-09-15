job "dns" {
  datacenters = ["dc1"]
  type        = "system"

  group "dnsmasq" {
    network {
      mode = "bridge"
      port "dns" {
        static = 53
        to     = 53
      }
    }


    task "dnsmasq" {
      driver = "docker"

      config {
        #This is an alpine-based dnsmasq container
        image = "4km3/dnsmasq:2.85-r2"
        ports = ["dns"]
        args = [
          # Send all queries for .consul to the NOMAD_IP
          "--server", "/consul/${NOMAD_IP_dns}#8600",
          # log to standard out
          "--log-facility=-",
        ]
        cap_add = [
          "NET_BIND_SERVICE",
        ]
        logging {
          type = "journald"
          config {
            tag = "DNSMASQ"
          }
        }
      }

      service {
        name         = "dnsmasq"
        port         = "dns"
        address_mode = "driver"
        tags         = ["dns"]

        check {
          type     = "tcp"
          port     = "dns"
          interval = "10s"
          timeout  = "2s"
        }
      }

      resources {
        cpu    = 50
        memory = 100
      }
    }
  }
}