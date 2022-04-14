# Cram

Small Rust CLI utility to compress/decompress data using Brotli, Gzip, LZ4, Snappy, or ZSTD algorithms.

## Build

`cargo build --release`

## Build & Run

`cargo run --release -- -f <filepath> -a <algorithm> -o <operation>`

## Examples

```
// compress a file with zstd
./cram -f example.json -a zstd -o compress >> example.zstd
```

```
// decompress a file with snappy
./cram -f example.snappy -a snappy -o decompress >> example.json
```

```
// run a compression benchmark on a file (25 runs per algorithm)
./cram -f example.json
```
