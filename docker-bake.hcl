# docker-bake.hcl
group "default" {
  targets = ["easysocks"]
}
target "easysocks" {
  dockerfile = "Dockerfile"
  tags       = ["latest", "${BUILD_NUMBER}"]
}