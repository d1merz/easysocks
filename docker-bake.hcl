# docker-bake.hcl
target "default" {
  dockerfile = "Dockerfile"
  tags       = ["latest", BUILD_NUMBER]
}