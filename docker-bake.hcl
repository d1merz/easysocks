# docker-bake.hcl
target "default" {
  dockerfile = "Dockerfile"
  tags       = ["latest", target.args.build_number]
}