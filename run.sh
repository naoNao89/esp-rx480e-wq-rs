#!/usr/bin/env bash
# Build, flash, and monitor the ESP32-C3 RX480-E-WQ firmware.
#
# Examples:
#   ./run.sh
#   PORT=/dev/cu.usbmodem11101 ./run.sh
#   NO_FLASH=1 ./run.sh
#   NO_MONITOR=1 ./run.sh
#   NO_LOG=1 ./run.sh
#   LOG_FILE=logs/rx480e.log ./run.sh

set -euo pipefail

if [ -f "$HOME/export-esp.sh" ]; then
  # shellcheck disable=SC1091
  . "$HOME/export-esp.sh"
fi

PORT="${PORT:-/dev/cu.usbmodem11101}"
BAUD="${BAUD:-115200}"
TARGET="${TARGET:-riscv32imc-unknown-none-elf}"
LOG_DIR="${LOG_DIR:-logs}"
LOG_FILE="${LOG_FILE:-}"
BIN="target/${TARGET}/release/esp32-c3-rx480e-wq"

if [ "${NO_LOG:-0}" != "1" ] && [ -z "$LOG_FILE" ]; then
  LOG_FILE="$LOG_DIR/rx480e-$(date +%Y%m%d-%H%M%S).log"
fi

echo "--- build release firmware for ${TARGET} ---"
env -u RUSTFLAGS cargo build -p esp32-c3-rx480e-wq --release --target "$TARGET"

if [ "${NO_FLASH:-0}" != "1" ]; then
  echo "--- flash ${PORT} ---"
  env -u RUSTFLAGS espflash flash --port "$PORT" "$BIN"
fi

if [ "${NO_MONITOR:-0}" = "1" ]; then
  exit 0
fi

if [ "${NO_LOG:-0}" != "1" ]; then
  mkdir -p "$(dirname "$LOG_FILE")"
fi

python3 - "$PORT" "$BAUD" "$LOG_FILE" <<'PY'
import sys
import time
from datetime import datetime

try:
    import serial
except ImportError:
    print("pyserial is required for monitor; install with: python3 -m pip install pyserial", file=sys.stderr)
    sys.exit(1)

port = sys.argv[1]
baud = int(sys.argv[2])
log_file = sys.argv[3] or None

print(f"--- monitoring {port} @{baud}, Ctrl-C to quit ---", flush=True)
if log_file:
    print(f"--- logging to {log_file} ---", flush=True)

def write_log(handle, text):
    if handle:
        handle.write(text)
        handle.flush()

log = open(log_file, "a", encoding="utf-8") if log_file else None
if log:
    write_log(log, f"\n--- log started {datetime.now().isoformat(timespec='seconds')} port={port} baud={baud} ---\n")

try:
    with serial.Serial(port, baud, timeout=0.5) as s:
        s.dtr = False
        s.rts = False
        time.sleep(0.1)
        while True:
            data = s.read(512)
            if data:
                text = data.decode("utf-8", errors="replace")
                print(text, end="", flush=True)
                write_log(log, text)
except KeyboardInterrupt:
    pass
except serial.SerialException as exc:
    message = f"\n--- serial disconnected: {exc} ---\n"
    print(message, end="", flush=True)
    write_log(log, message)
finally:
    if log:
        write_log(log, f"--- log ended {datetime.now().isoformat(timespec='seconds')} ---\n")
        log.close()
PY
