# srvcs-abs

## Name

| Field | Value |
| --- | --- |
| Service | `srvcs-abs` |
| Slug | `abs` |
| Repository | `srvcs/abs` |
| Package | `srvcs-abs` |
| Kind | `orchestrator` |

## Function

absolute value

## Dependencies

| Dependency | Repository |
| --- | --- |
| `srvcs-isnegative` | [srvcs/isnegative](https://github.com/srvcs/isnegative) |
| `srvcs-negate` | [srvcs/negate](https://github.com/srvcs/negate) |

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity |
| `POST` | `/` | Evaluate the service function |
| `GET` | `/healthz` | Liveness probe |
| `GET` | `/readyz` | Readiness probe |
| `GET` | `/metrics` | Prometheus metrics |
| `GET` | `/openapi.json` | OpenAPI document |

## Inputs

| Name | Type | Required |
| --- | --- | --- |
| `value` | `json` | yes |

## Outputs

| Name | Type |
| --- | --- |
| `value` | `json` |
| `result` | `json` |

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |
| `SRVCS_ISNEGATIVE_URL` | `http://127.0.0.1:8084` | Base URL for srvcs-isnegative |
| `SRVCS_NEGATE_URL` | `http://127.0.0.1:8085` | Base URL for srvcs-negate |

## Error Behavior

- `422` means the request could not be evaluated for the documented input shape.
- `503` means a required dependency was unavailable or returned an unexpected response.
- Dependency validation errors are forwarded when this service delegates validation.

## Local Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

See the [srvcs service standard](https://github.com/srvcs/platform/blob/main/STANDARD.md) for the full operational contract.

## Metadata

Machine-readable service metadata lives in `srvcs.yaml`. Keep it aligned with this README when the service contract changes.
