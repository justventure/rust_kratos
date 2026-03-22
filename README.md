# Kratos rust example service
<div align="left">
  <!-- Dependencies & security -->
  <a href="https://deps.rs/repo/github/vwency/rust_kratos"><img src="https://deps.rs/repo/github/vwency/rust_kratos/status.svg"/></a>
  <!-- Rust -->
  <img src="https://img.shields.io/badge/rust-1.95.0--nightly-orange?logo=rust"/>
  <img src="https://img.shields.io/badge/unsafe-forbidden-success?logo=rust"/>
  <!-- Meta -->
  <a href="https://github.com/vwency/rust_kratos/blob/main/LICENSE"><img src="https://img.shields.io/github/license/vwency/rust_kratos"/></a>
  <img src="https://img.shields.io/github/last-commit/vwency/rust_kratos"/>
</div>

### Execute
```
make infra-up
make run
```

### Available APP_ENV values
| APP_ENV value | Используемый конфиг |
|---------------|---------------------|
| development   | development.toml    |
| production    | production.toml     |
| docker_local  | docker_local.toml   |
| -             | development.toml    |

### Benchmark
![Benchmark](benchmark/Screenshot%20From%202026-03-23%2002-22-38.png)
