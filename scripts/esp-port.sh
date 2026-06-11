#!/usr/bin/env bash
# Resolve the ESP serial port safely.
#
# Usage:
#   ./scripts/esp-port.sh --print
#   ./scripts/esp-port.sh command using "$ESP_PORT"
#
# If PORT is set, it is used as-is. Otherwise, exactly one likely USB serial
# device must be present. Multiple candidates require explicit PORT=... to avoid
# flashing or monitoring the wrong device.

set -euo pipefail

resolve_port() {
  if [ -n "${PORT:-}" ]; then
    printf '%s\n' "$PORT"
    return 0
  fi

  local candidates=()
  local pattern port
  for pattern in \
    /dev/cu.usbmodem* \
    /dev/cu.usbserial* \
    /dev/cu.wchusbserial* \
    /dev/cu.SLAB_USBtoUART* \
    /dev/ttyACM* \
    /dev/ttyUSB*
  do
    for port in $pattern; do
      [ -e "$port" ] || continue
      candidates+=("$port")
    done
  done

  case "${#candidates[@]}" in
    0)
      cat >&2 <<'EOF'
No likely ESP serial port found.

Connect the board, then retry or specify a port explicitly, for example:
  make flash PORT=/dev/cu.usbmodem11301
  make monitor PORT=/dev/cu.usbmodem11301
EOF
      return 1
      ;;
    1)
      printf '%s\n' "${candidates[0]}"
      ;;
    *)
      cat >&2 <<'EOF'
Multiple likely serial ports found:
EOF
      printf '  %s\n' "${candidates[@]}" >&2
      cat >&2 <<'EOF'

Specify the ESP port explicitly, for example:
  make flash PORT=/dev/cu.usbmodem11301
  make monitor PORT=/dev/cu.usbmodem11301
EOF
      return 1
      ;;
  esac
}

ESP_PORT="$(resolve_port)"
export ESP_PORT

if [ "${1:-}" = "--print" ]; then
  printf '%s\n' "$ESP_PORT"
  exit 0
fi

if [ "$#" -eq 0 ]; then
  printf '%s\n' "$ESP_PORT"
  exit 0
fi

exec "$@"
