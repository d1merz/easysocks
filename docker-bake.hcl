# docker-bake.hcl
variable "BUILD_NUMBER" {
  default = "v1"
}
group "default" {
  targets = ["easysocks"]
}
target "easysocks" {
  dockerfile = "Dockerfile"
  tags       = ["latest", "${BUILD_NUMBER}"]
}