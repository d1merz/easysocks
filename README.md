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
docker run -v /absolute/path/to/users.csv:/easysocks/users.csv --proto tcp --port 8001 --ip 127.0.0.1 --users users.csv
```



