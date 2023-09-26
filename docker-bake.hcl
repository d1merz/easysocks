# docker-bake.hcl
target "default" {
  args = {
    BUILD_NUMBER = null
  }
  dockerfile = "Dockerfile"
  tags       = ["latest", target.args.BUILD_NUMBER]
}