# docker-bake.hcl
target "default" {
  variable "BUILD_NUMBER" {
    default = "v1"
  }
  dockerfile = "Dockerfile"
  tags       = ["latest", BUILD_NUMBER]
}