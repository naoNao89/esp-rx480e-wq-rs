PORT ?= /dev/cu.usbmodem11101
BAUD ?= 115200
TARGET ?= riscv32imc-unknown-none-elf
HOST_TARGET ?= $(shell rustc -vV | awk '/^host:/ { print $$2 }')
LOG_DIR ?= logs
LOG_FILE ?=
NO_FLASH ?= 0
NO_MONITOR ?= 0
NO_LOG ?= 0
BIN := target/$(TARGET)/release/esp32-c3-rx480e-wq

.PHONY: test build flash monitor run clean fmt check

test:
	cargo test -p rx480e-wq-driver --target $(HOST_TARGET)

build:
	env -u RUSTFLAGS cargo build -p esp32-c3-rx480e-wq --release --target $(TARGET)

flash: build
	env -u RUSTFLAGS espflash flash --port "$(PORT)" "$(BIN)"

monitor:
	PORT="$(PORT)" BAUD="$(BAUD)" TARGET="$(TARGET)" LOG_DIR="$(LOG_DIR)" LOG_FILE="$(LOG_FILE)" NO_FLASH=1 NO_MONITOR="$(NO_MONITOR)" NO_LOG="$(NO_LOG)" ./run.sh

run:
	PORT="$(PORT)" BAUD="$(BAUD)" TARGET="$(TARGET)" LOG_DIR="$(LOG_DIR)" LOG_FILE="$(LOG_FILE)" NO_FLASH="$(NO_FLASH)" NO_MONITOR="$(NO_MONITOR)" NO_LOG="$(NO_LOG)" ./run.sh

clean:
	cargo clean

fmt:
	cargo fmt

check:
	cargo fmt -- --check
	cargo check --workspace
