# rx480e-wq-driver

`no_std` `embedded-hal` driver helpers for RX480-E-WQ / RX480E-4 receiver modules.

The crate is intentionally small: it samples the module's active-high `D0`-`D3` channel outputs and `VT` valid-transmission output, then reports snapshots and edge changes.

```rust
let mut rx = rx480e_wq_driver::Rx480eWq::new(d0, d1, d2, d3, vt);

if let Some(event) = rx.poll_change()? {
    if event.vt_rising() {
        // valid frame started
    }

    match event.current.channel_state() {
        rx480e_wq_driver::ChannelState::Single(channel) => {
            // D0, D1, D2, or D3 is active
        }
        rx480e_wq_driver::ChannelState::None => {
            // no channel output is active
        }
        rx480e_wq_driver::ChannelState::Multiple(mask) => {
            // multiple channel outputs are active
        }
    }
}
```

Limitations:

- Active-high RX480 outputs are assumed.
- Learned-code memory is internal to the RX480 module and cannot be read through this driver.
- Pulse timing is board-specific; use your MCU timer or polling period outside this crate.
