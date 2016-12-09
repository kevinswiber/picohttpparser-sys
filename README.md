# picohttpparser-sys

* Rust bindings for [picohttpparser](https://github.com/h2o/picohttpparser).

## Building

To increase performance, enable SSE 4.2 when building projects that depend on `picohttpparser-sys`.

Example: `RUSTFLAGS="-C target-feature=+sse4.2" cargo build`

To enable this feature by default on all cargo builds, you can set this feature in `~/cargo/config` :

```toml
[build]
rustflags = ["-C", "target-feature=+sse4.2"]
```

## License

MIT
