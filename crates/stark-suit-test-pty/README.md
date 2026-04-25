# stark-suit-test-pty

Pseudoterminal (PTY) loopback fixture for `SerialBmsSource` integration tests.

## Why this exists

`SerialBmsSource` opens a serial-class device path (`/dev/ttyUSB0`, `/dev/ttyACM0`, ...). Without hardware, the trait abstraction is unverified — the third leg of the BmsSource ρ contract is open.

This crate spawns a kernel master/slave PTY pair via `nix::pty::openpty`. The slave path (`/dev/pts/N`) is a real character device the kernel hands `tokio-serial::open_native_async` exactly as it would `/dev/ttyUSB0`. Tests write NDJSON to the master end; `SerialBmsSource` reads it back through the slave. Round-trip equality validates the protocol layer of the trait.

## Why nix, not socat

| | `nix::pty` | `socat` |
|---|---|---|
| Dependency | one Rust crate | system binary |
| CI portability | works anywhere `cargo test` works | requires `apt install socat` preflight |
| Path discovery | `nix::unistd::ttyname()` returns slave path | symlink dance with `link=` flag |
| Teardown | `Drop` closes fd, kernel reaps pair | extra `kill()` in test cleanup |
| Unsafe | zero — nix wraps `openpty(3)` safely | n/a (subprocess) |

Trade-off: nix is Linux + macOS only. Windows uses ConPTY, a different API. Out of scope — BMS hardware does not run on Windows.

## Usage

```rust
use stark_suit_test_pty::TestPty;

let mut pty = TestPty::spawn().expect("openpty");
let slave_path = pty.slave_path.to_str().unwrap();

// Open the slave path with tokio-serial as if it were /dev/ttyUSB0.
let serial = SerialBmsSource::open(slave_path, 115200)?;

// Push a frame through the master.
pty.write_line(r#"{"version":1,...}"#).await?;

// Read it back through the SerialBmsSource.
let frame = serial.poll().await?;
```

## What this proves vs. what it does not

| | proves | does NOT prove |
|---|---|---|
| Trait abstraction | yes — three backends pass the same contract | — |
| NDJSON framing across byte stream | yes | — |
| `tokio-serial::open_native_async` consumer path | yes | — |
| Baud rate sync against real silicon | — | needs USB-serial adapter |
| USB hot-unplug, electrical noise | — | needs hardware |
| CAN-vs-UART framing differences | — | out of scope (CAN deferred to v0.5+) |

The pty test closes the protocol-fidelity ρ leg. A real `/dev/ttyUSB0` test (same code, different path argument) will close the physical-fidelity leg when hardware arrives.
