job "some-job" {
  datacenters = ["dc1"]
  type        = "service"

  group "some-group" {
    task "some-task" {
      driver = "docker"

      config {
        image = "fake-image-just-for-nomad-job-plan"
      }

      resources {
        # Seems unlikely we'd be running this test on a machinge with 1TB RAM any time soon
        memory = 1000 * 1000
      }
    }
  }
}