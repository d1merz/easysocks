# docker-bake.hcl
variable "BUILD_NUMBER" {
  default = "1"
}
group "default" {
  targets = ["easysocks"]
}
target "easysocks" {
  dockerfile = "Dockerfile"
  tags       = ["ghcr.io/dimerz-tech/easysocks:${BUILD_NUMBER}", "ghcr.io/dimerz-tech/easysocks:latest"]
}