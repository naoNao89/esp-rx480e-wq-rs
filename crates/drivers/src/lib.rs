#![no_std]

use embedded_hal::digital::InputPin;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Channel {
    D0,
    D1,
    D2,
    D3,
    VT,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub struct Snapshot {
    pub d0: bool,
    pub d1: bool,
    pub d2: bool,
    pub d3: bool,
    pub vt: bool,
}

impl Snapshot {
    pub fn get(&self, channel: Channel) -> bool {
        match channel {
            Channel::D0 => self.d0,
            Channel::D1 => self.d1,
            Channel::D2 => self.d2,
            Channel::D3 => self.d3,
            Channel::VT => self.vt,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event {
    pub previous: Snapshot,
    pub current: Snapshot,
}

pub struct Rx480eWq<D0, D1, D2, D3, VT> {
    d0: D0,
    d1: D1,
    d2: D2,
    d3: D3,
    vt: VT,
    last: Snapshot,
}

impl<D0, D1, D2, D3, VT> Rx480eWq<D0, D1, D2, D3, VT>
where
    D0: InputPin,
    D1: InputPin<Error = D0::Error>,
    D2: InputPin<Error = D0::Error>,
    D3: InputPin<Error = D0::Error>,
    VT: InputPin<Error = D0::Error>,
{
    pub fn new(d0: D0, d1: D1, d2: D2, d3: D3, vt: VT) -> Self {
        Self {
            d0,
            d1,
            d2,
            d3,
            vt,
            last: Snapshot::default(),
        }
    }

    pub fn sample(&mut self) -> Result<Snapshot, D0::Error> {
        Ok(Snapshot {
            d0: self.d0.is_high()?,
            d1: self.d1.is_high()?,
            d2: self.d2.is_high()?,
            d3: self.d3.is_high()?,
            vt: self.vt.is_high()?,
        })
    }

    pub fn poll_change(&mut self) -> Result<Option<Event>, D0::Error> {
        let current = self.sample()?;
        if current == self.last {
            return Ok(None);
        }
        let previous = self.last;
        self.last = current;
        Ok(Some(Event { previous, current }))
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[derive(Clone)]
    struct MockPin {
        state: Cell<bool>,
    }

    impl MockPin {
        fn new(state: bool) -> Self {
            Self {
                state: Cell::new(state),
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

    #[test]
    fn sample_reads_all_channels() {
        let mut dev = Rx480eWq::new(
            MockPin::new(true),
            MockPin::new(false),
            MockPin::new(true),
            MockPin::new(false),
            MockPin::new(true),
        );
        let s = dev.sample().unwrap();
        assert_eq!(
            s,
            Snapshot {
                d0: true,
                d1: false,
                d2: true,
                d3: false,
                vt: true
            }
        );
    }

    #[test]
    fn poll_change_emits_event_when_state_changes() {
        let mut dev = Rx480eWq::new(
            MockPin::new(false),
            MockPin::new(false),
            MockPin::new(false),
            MockPin::new(false),
            MockPin::new(false),
        );
        assert!(dev.poll_change().unwrap().is_none());
        dev.d0.state.set(true);
        let evt = dev.poll_change().unwrap().expect("event");
        assert_eq!(evt.previous.d0, false);
        assert_eq!(evt.current.d0, true);
    }
}
