# rx480e-wq-driver

`no_std` `embedded-hal` driver helpers for RX480-E-WQ / RX480E-4 receiver modules.

The crate is intentionally small: it samples the module's active-high `D0`-`D3` channel outputs and `VT` valid-transmission output, then reports snapshots and edge changes.

It does not print serial logs by itself. Logs such as `EVENT: key=D0 vt=1 pulse_ms=...` belong to application firmware, not to this driver crate.

Layer split:

```text
Driver crate:    Snapshot / Event / ChannelState
Application:     serial logs, pulse timing, relay control, Home Assistant messages, etc.
```

```rust
let mut rx = rx480e_wq_driver::Rx480eWq::new(d0, d1, d2, d3, vt);

if let Some(event) = rx.poll_change()? {
    if event.vt_rising() {
        // valid frame started
    }

    match event.current.channel_state() {
        rx480e_wq_driver::ChannelState::Single(channel) => {
            // D0, D1, D2, or D3 is active.
            // VT is not returned as a ChannelState; use event.vt_rising(),
            // event.vt_falling(), or snapshot.vt_only() for VT state.
        }
        rx480e_wq_driver::ChannelState::None if event.current.vt_only() => {
            // VT active, but no D0-D3 channel output is active.
        }
        rx480e_wq_driver::ChannelState::None => {
            // no channel output is active.
        }
        rx480e_wq_driver::ChannelState::Multiple(mask) => {
            // multiple channel outputs are active.
        }
    }
}
```

Limitations:

- Active-high RX480 outputs are assumed.
- Learned-code memory is internal to the RX480 module and cannot be read through this driver.
- Pulse timing is board-specific; use your MCU timer or polling period outside this crate.
- Learning, clearing, transmitting, replaying RF, serial logging, firmware flashing, and pulse-duration measurement are application/board-firmware responsibilities.
