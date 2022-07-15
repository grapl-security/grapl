variable "string_var" {
  type        = string
  description = "Sample variable with string type."
}

variable "number_var" {
  type        = number
  description = "Sample variable with number type."
}

variable "map_string_var" {
  type        = map(string)
  description = "Sample variable with a map of strings type."
}

variable "object_var" {
  type = object({
    hostname = string
    port     = number
    username = string
    password = string
  })
  description = "Sample variable with an object type."
}

variable "map_object_var" {
  description = "Sample variable with a map of objects type."
  type = map(object({
    # The username to authenticate with Confluent Cloud cluster.
    sasl_username = string
    # The password to authenticate with Confluent Cloud cluster.
    sasl_password = string
  }))
}


