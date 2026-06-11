use std::{cell::Cell, rc::Rc};

use embedded_hal::digital::InputPin;
use rx480e_wq_driver::{ChannelState, Rx480eWq, Signal};

#[derive(Clone)]
struct MockPin {
    state: Rc<Cell<bool>>,
}

impl MockPin {
    fn new(state: bool) -> Self {
        Self {
            state: Rc::new(Cell::new(state)),
        }
    }
}

impl embedded_hal::digital::ErrorType for MockPin {
    type Error = core::convert::Infallible;
}

impl InputPin for MockPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.state.get())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.state.get())
    }
}

fn main() {
    let d0 = MockPin::new(false);
    let d1 = MockPin::new(false);
    let d2 = MockPin::new(false);
    let d3 = MockPin::new(false);
    let vt = MockPin::new(false);

    let d0_handle = d0.clone();
    let vt_handle = vt.clone();
    let mut rx = Rx480eWq::new(d0, d1, d2, d3, vt);

    assert_eq!(rx.poll_change(), Ok(None));

    vt_handle.state.set(true);
    d0_handle.state.set(true);

    if let Some(event) = rx.poll_change().expect("sample pins") {
        if event.vt_rising() {
            println!("valid transmission started");
        }

        if event.edge(Signal::D0).is_some() {
            println!("D0 changed");
        }

        match event.current.channel_state() {
            ChannelState::Single(channel) => println!("{} active", channel.name()),
            ChannelState::None if event.current.vt_only() => println!("VT only"),
            ChannelState::Multiple(mask) => println!("multiple channels: 0x{mask:x}"),
            ChannelState::None => println!("no channel active"),
        }
    }
}
