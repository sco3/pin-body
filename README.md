# pin-body

A Pingora-based HTTP proxy with request body inspection, adapted from [Lesson 12: Body Inspection](https://dev.to/warren_jitsing_dd1c1d6fc6/pingora-guide-how-to-make-a-programmable-api-gateway-1oim#lesson-12-body-inspection) of the Pingora API Gateway guide.

The proxy buffers incoming request bodies, scans for the forbidden keyword `"rogue"`, and blocks the request with a `SecurityPolicyViolation` error if found. Allowed requests are forwarded to a local upstream server written in Go.

---

## What's Different from the Article

### 1. `BytesMut` buffer instead of `Vec<u8>`

| Article | This project |
|---|---|
| `buffer: Vec<u8>` | `buffer: BytesMut` |

The article uses a plain `Vec<u8>` to accumulate body chunks. This project uses [`BytesMut`](https://docs.rs/bytes/latest/bytes/struct.BytesMut.html) from the `bytes` crate — the same type that Pingora uses internally for body chunks. This avoids unnecessary copies and integrates more naturally with the Pingora API.

When forwarding the buffered body back to the upstream, the article doesn't show how to put the data back. This project uses `ctx.buffer.split().freeze()` to convert the `BytesMut` into an immutable `Bytes` value and assign it back to `*body` in a single zero-copy operation:

```rust
*body = Some(ctx.buffer.split().freeze());
```

---

### 2. Deferred inspection — checks only at end of stream

| Article | This project |
|---|---|
| Checks for `"rogue"` on **every chunk** as it arrives | Checks for `"rogue"` **only after all chunks are buffered** (`end_of_stream == true`) |

The article runs `content.contains("rogue")` inside the chunk loop — meaning it scans the partial buffer after each incoming chunk. This project waits until `end_of_stream` is `true` before running the check:

```rust
if end_of_stream {
    BodyInspector::check_body(&ctx.buffer).await?;
    *body = Some(ctx.buffer.split().freeze());
}
```

This is a deliberate trade-off: the full body is assembled first, so the scan is performed exactly once on the complete payload. It also eliminates false negatives where the word `"rogue"` could be split across two chunks.

---

### 3. `memchr` for pattern matching instead of `String::from_utf8_lossy`

| Article | This project |
|---|---|
| `String::from_utf8_lossy(&ctx.buffer).contains("rogue")` | `memchr::memmem::find(body, b"rogue").is_some()` |

The article converts the buffer to a string (with lossy UTF-8 decoding) before calling `.contains()`. This project uses the [`memchr`](https://docs.rs/memchr/latest/memchr/) crate's `memmem::find`, which performs a direct byte-level substring search using a highly optimised algorithm (SIMD-accelerated on supported platforms). This:

- Avoids UTF-8 conversion overhead entirely
- Works correctly on binary bodies
- Is faster for large payloads

---

### 4. `check_body` extracted into a dedicated `async fn`

The article puts all inspection logic inline inside `request_body_filter`. This project extracts the scan into a separate associated function:

```rust
impl BodyInspector {
    async fn check_body(body: &BytesMut) -> Result<()> {
        if memchr::memmem::find(body, b"rogue").is_some() {
            return Err(pingora::Error::new(ErrorType::Custom(
                "SecurityPolicyViolation",
            )));
        }
        Ok(())
    }
}
```

This keeps `request_body_filter` focused on buffering logic and makes the security check easy to extend or test independently.

---

### 5. Local upstream instead of Docker "Pingora City"

| Article | This project |
|---|---|
| Upstream: Docker container at `172.28.0.20:8080` | Upstream: `localhost:8080` |
| SNI hostname: `"blue.pingora.local"` | SNI hostname: `""` (plain HTTP, no TLS) |

The article is designed around a Docker Compose lab ("Pingora City"). This project is self-contained: a lightweight Go HTTP server (`server/main.go`) acts as the upstream, running locally on port `8080`. No Docker setup is needed.

The Go server echoes the received body back as JSON:

```json
{ "status": "ok", "received": "<body content>" }
```

---

### 6. Proxy listens on port `6152`

The proxy binds to `0.0.0.0:6152` (same as the article).

---

## Architecture

```
Client
  │
  │  HTTP POST
  ▼
Pingora Proxy (port 6152)         ← src/main.rs
  │
  │  buffer all chunks
  │  scan for "rogue" with memchr
  │  block → SecurityPolicyViolation
  │  allow → forward complete body
  ▼
Go upstream server (port 8080)    ← server/main.go
  │
  │  { "status": "ok", "received": "..." }
  ▼
Client
```

---

## Running

### 1. Start the upstream Go server

```bash
cd server
go run main.go
```

### 2. Start the proxy

```bash
RUST_LOG=info cargo run
```

### 3. Test a clean request

```bash
curl -X POST -d "Hello World" http://localhost:6152
# {"status":"ok","received":"Hello World"}
```

### 4. Test a blocked request

```bash
curl -X POST -d "I am a rogue agent" http://localhost:6152
# 500 Internal Server Error
```

Proxy log output for a blocked request:

```
ERROR Fail to proxy: SecurityPolicyViolation ...
```

---

## Dependencies

| Crate | Purpose |
|---|---|
| `pingora` | Proxy framework |
| `bytes` | `BytesMut` / `Bytes` for zero-copy body buffering |
| `memchr` | Fast byte-level substring search |
| `async-trait` | `async fn` in traits |
| `log` + `env_logger` | Logging |
| `tokio` | Async runtime |