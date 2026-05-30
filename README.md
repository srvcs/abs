# srvcs-abs

The absolute-value orchestrator of the srvcs.cloud distributed standard library.

Its single concern: **the absolute value of a number.** It does no arithmetic of
its own. It asks [`srvcs-isnegative`](https://github.com/srvcs/isnegative)
whether `value` is negative; if so, it asks
[`srvcs-negate`](https://github.com/srvcs/negate) to flip the sign and returns
that. Otherwise the value is already its own absolute value.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity, concern, and dependency list |
| `POST` | `/` | Absolute value of `value` |
| `GET` | `/healthz` `/readyz` `/metrics` `/openapi.json` | srvcs service standard surface |

```sh
curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"value": -6}'
# {"value":-6,"result":6}
```

Responses:

- `200 {"value": n, "result": |n|}` — evaluated.
- `422` — invalid input, forwarded from a leaf dependency.
- `503` — a dependency is unavailable.

## Dependencies

- [`srvcs-isnegative`](https://github.com/srvcs/isnegative)
- [`srvcs-negate`](https://github.com/srvcs/negate)

Input validation propagates from the leaf dependencies via their `422`
responses — this service does not depend on `srvcs-isnumber` directly.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ISNEGATIVE_URL` | `http://127.0.0.1:8084` | Base URL of `srvcs-isnegative` |
| `SRVCS_NEGATE_URL` | `http://127.0.0.1:8085` | Base URL of `srvcs-negate` |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |

## Local checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Orchestration tests stand up mock `srvcs-isnegative` and `srvcs-negate` services
in-process, including the degraded (`503`) and forwarded-invalid-input (`422`)
cases. See [`srvcs/platform`](https://github.com/srvcs/platform) for the shared
standard.

> Note: the `cargoHash` in `flake.nix` is inherited from the template and must be
> refreshed with a `nix build` before the Nix gates pass.
