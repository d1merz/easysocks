## Simple and lightweight `SOCKS5` server implementation in Rust

![github actions workflow](https://github.com/dimerz-tech/easysocks/actions/workflows/rust.yml/badge.svg)

* 📝 **100% according to [RFC 1928](https://datatracker.ietf.org/doc/html/rfc1928)**
* 🔎 **Clean code**
* 📈 **High performance and robust**
* ✅ **Tokio + Serde = No environment/system dependencies required**

## Run server

### Run binary
```bash
# Compile
cargo build --release
# Run
target/release/easysocks --proto tcp --port 8001 --ip 127.0.0.1 --users users.csv
```

*users.csv example*:
```csv
name,pass
admin,admin
```

### Run docker container
```bash
docker pull ghcr.io/dimerz-tech/easysocks:latest
docker run -v path/to/users.csv:/users.csv ghcr.io/dimerz-tech/easysocks:latest --proto tcp --port 8001 --ip 127.0.0.1 --users users.csv
```
or use `docker-compose`:
```bash
# with docker-compose.yml and users.csv
docker-compose up
```







