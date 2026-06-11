//! `no_std` [`embedded-hal`](embedded_hal) input helper for RX480-E-WQ / RX480E-4
//! decoded output pins.
//!
//! This crate reads the module's active-high `D0`-`D3` decoded channel outputs
//! and `VT` valid-transmission output through [`embedded_hal::digital::InputPin`].
//!
//! Pin conventions:
//! - `D0`-`D3` are active-high decoded channel outputs.
//! - `VT` is active-high and indicates a valid transmission.
//! - Bits are laid out as `D0=0b0000_0001`, `D1=0b0000_0010`, `D2=0b0000_0100`,
//!   `D3=0b0000_1000`, `VT=0b0001_0000`.
//! - Sampling is sequential (`D0`, `D1`, `D2`, `D3`, `VT`), not atomic.
//! - `poll_change` compares against an initial all-low baseline, so a pin that
//!   is already high on the first call is reported as a rising edge.
//!
//! It reports structured [`Snapshot`] and [`Event`] values that application
//! firmware can turn into logs, relay actions, MQTT messages, or other behavior.
//!
//! It does **not** decode raw RF, pair remotes, clear learned codes, inspect the
//! RX480 module's internal learned-code memory, transmit RF, or measure pulse
//! durations.
//!
//! ```no_run
//! use rx480e_wq_driver::{ChannelState, Rx480eWq};
//!
//! # fn run<D0, D1, D2, D3, VT>(d0: D0, d1: D1, d2: D2, d3: D3, vt: VT) -> Result<(), D0::Error>
//! # where
//! #     D0: embedded_hal::digital::InputPin,
//! #     D1: embedded_hal::digital::InputPin<Error = D0::Error>,
//! #     D2: embedded_hal::digital::InputPin<Error = D0::Error>,
//! #     D3: embedded_hal::digital::InputPin<Error = D0::Error>,
//! #     VT: embedded_hal::digital::InputPin<Error = D0::Error>,
//! # {
//! let mut rx = Rx480eWq::new(d0, d1, d2, d3, vt);
//!
//! if let Some(event) = rx.poll_change()? {
//!     if event.vt_rising() {
//!         // A valid RF frame was accepted by the RX480 module.
//!     }
//!
//!     match event.current.channel_state() {
//!         ChannelState::Single(channel) => {
//!             // One of D0-D3 is active. `VT` is handled separately.
//!             let _name = channel.name();
//!         }
//!         ChannelState::None if event.current.vt_only() => {
//!             // VT is active, but no D0-D3 output is active.
//!         }
//!         ChannelState::None => {}
//!         ChannelState::Multiple(mask) => {
//!             // Multiple D0-D3 outputs are active.
//!             let _mask = mask;
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

#![no_std]

use embedded_hal::digital::InputPin;

/// Bit for the active-high `D0` channel output.
pub const D0_BIT: u8 = 0b0000_0001;
/// Bit for the active-high `D1` channel output.
pub const D1_BIT: u8 = 0b0000_0010;
/// Bit for the active-high `D2` channel output.
pub const D2_BIT: u8 = 0b0000_0100;
/// Bit for the active-high `D3` channel output.
pub const D3_BIT: u8 = 0b0000_1000;
/// Bit for the active-high `VT` valid-transmission output.
pub const VT_BIT: u8 = 0b0001_0000;

/// One decoded channel output (`D0`-`D3`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Channel {
    D0,
    D1,
    D2,
    D3,
}

impl Channel {
    pub fn name(self) -> &'static str {
        match self {
            Channel::D0 => "D0",
            Channel::D1 => "D1",
            Channel::D2 => "D2",
            Channel::D3 => "D3",
        }
    }

    pub fn bit(self) -> u8 {
        match self {
            Channel::D0 => D0_BIT,
            Channel::D1 => D1_BIT,
            Channel::D2 => D2_BIT,
            Channel::D3 => D3_BIT,
        }
    }
}

/// One sampled signal (`D0`-`D3` or `VT`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Signal {
    D0,
    D1,
    D2,
    D3,
    VT,
}

impl Signal {
    pub fn name(self) -> &'static str {
        match self {
            Signal::D0 => "D0",
            Signal::D1 => "D1",
            Signal::D2 => "D2",
            Signal::D3 => "D3",
            Signal::VT => "VT",
        }
    }

    pub fn bit(self) -> u8 {
        match self {
            Signal::D0 => D0_BIT,
            Signal::D1 => D1_BIT,
            Signal::D2 => D2_BIT,
            Signal::D3 => D3_BIT,
            Signal::VT => VT_BIT,
        }
    }
}

/// Decoded `D0`-`D3` channel state.
///
/// `VT` is not returned as a `ChannelState`; use [`Snapshot::vt_only`],
/// [`Event::vt_rising`], or [`Event::vt_falling`] for valid-transmission state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelState {
    None,
    Single(Channel),
    Multiple(u8),
}

/// Rising/falling transition on a sampled signal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Edge {
    Rising,
    Falling,
}

/// Classify only the `D0`-`D3` channel bits.
pub fn channel_state_from_channel_bits(bits: u8) -> ChannelState {
    match bits & (D0_BIT | D1_BIT | D2_BIT | D3_BIT) {
        D0_BIT => ChannelState::Single(Channel::D0),
        D1_BIT => ChannelState::Single(Channel::D1),
        D2_BIT => ChannelState::Single(Channel::D2),
        D3_BIT => ChannelState::Single(Channel::D3),
        0 => ChannelState::None,
        mask => ChannelState::Multiple(mask),
    }
}

#[deprecated(note = "use channel_state_from_channel_bits")]
pub fn channel_state_from_bits(bits: u8) -> ChannelState {
    channel_state_from_channel_bits(bits)
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
        }
    }

    pub fn get_signal(&self, signal: Signal) -> bool {
        match signal {
            Signal::D0 => self.d0,
            Signal::D1 => self.d1,
            Signal::D2 => self.d2,
            Signal::D3 => self.d3,
            Signal::VT => self.vt,
        }
    }

    /// Packed `D0`-`D3` bits in the low nibble.
    pub fn channel_bits(&self) -> u8 {
        (self.d0 as u8) | ((self.d1 as u8) << 1) | ((self.d2 as u8) << 2) | ((self.d3 as u8) << 3)
    }

    /// Returns true when at least one `D0`-`D3` channel output is active.
    pub fn any_channel_active(&self) -> bool {
        self.channel_bits() != 0
    }

    /// Packed snapshot bits: channels in the low nibble, `VT` in bit 4.
    pub fn bits(&self) -> u8 {
        self.channel_bits() | ((self.vt as u8) << 4)
    }

    /// Returns true when the `VT` valid-transmission output is active.
    pub fn is_valid_transmission(&self) -> bool {
        self.vt
    }

    pub fn active_channel(&self) -> Option<Channel> {
        match channel_state_from_channel_bits(self.channel_bits()) {
            ChannelState::Single(channel) => Some(channel),
            _ => None,
        }
    }

    pub fn channel_state(&self) -> ChannelState {
        channel_state_from_channel_bits(self.channel_bits())
    }

    /// Returns true when `VT` is active but no `D0`-`D3` channel output is active.
    pub fn vt_only(&self) -> bool {
        self.vt && self.channel_bits() == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Event {
    pub previous: Snapshot,
    pub current: Snapshot,
}

impl Event {
    pub fn changed_mask(&self) -> u8 {
        self.previous.bits() ^ self.current.bits()
    }

    pub fn edge(&self, signal: Signal) -> Option<Edge> {
        match (
            self.previous.get_signal(signal),
            self.current.get_signal(signal),
        ) {
            (false, true) => Some(Edge::Rising),
            (true, false) => Some(Edge::Falling),
            _ => None,
        }
    }

    pub fn vt_rising(&self) -> bool {
        self.edge(Signal::VT) == Some(Edge::Rising)
    }

    pub fn vt_falling(&self) -> bool {
        self.edge(Signal::VT) == Some(Edge::Falling)
    }
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

    /// Sample all pins and return an event when the snapshot differs from the
    /// previous sample.
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

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct PinError;

    impl embedded_hal::digital::Error for PinError {
        fn kind(&self) -> embedded_hal::digital::ErrorKind {
            embedded_hal::digital::ErrorKind::Other
        }
    }

    struct ErrorPin;

    impl embedded_hal::digital::ErrorType for ErrorPin {
        type Error = PinError;
    }

    impl InputPin for ErrorPin {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Err(PinError)
        }

        fn is_low(&mut self) -> Result<bool, Self::Error> {
            Err(PinError)
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
    fn channel_helpers_return_stable_names_and_bits() {
        assert_eq!(Channel::D0.name(), "D0");
        assert_eq!(Channel::D1.name(), "D1");
        assert_eq!(Channel::D2.name(), "D2");
        assert_eq!(Channel::D3.name(), "D3");

        assert_eq!(Channel::D0.bit(), D0_BIT);
        assert_eq!(Channel::D1.bit(), D1_BIT);
        assert_eq!(Channel::D2.bit(), D2_BIT);
        assert_eq!(Channel::D3.bit(), D3_BIT);
    }

    #[test]
    fn signal_helpers_return_stable_names_and_bits() {
        assert_eq!(Signal::D0.name(), "D0");
        assert_eq!(Signal::D1.name(), "D1");
        assert_eq!(Signal::D2.name(), "D2");
        assert_eq!(Signal::D3.name(), "D3");
        assert_eq!(Signal::VT.name(), "VT");

        assert_eq!(Signal::D0.bit(), D0_BIT);
        assert_eq!(Signal::D1.bit(), D1_BIT);
        assert_eq!(Signal::D2.bit(), D2_BIT);
        assert_eq!(Signal::D3.bit(), D3_BIT);
        assert_eq!(Signal::VT.bit(), VT_BIT);
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
        assert!(!evt.previous.d0);
        assert!(evt.current.d0);
    }

    #[test]
    fn snapshot_helpers_classify_channels() {
        let single = Snapshot {
            d0: false,
            d1: true,
            d2: false,
            d3: false,
            vt: true,
        };
        assert_eq!(single.channel_bits(), D1_BIT);
        assert!(single.any_channel_active());
        assert_eq!(single.bits(), D1_BIT | VT_BIT);
        assert!(single.is_valid_transmission());
        assert_eq!(single.active_channel(), Some(Channel::D1));
        assert_eq!(single.channel_state(), ChannelState::Single(Channel::D1));
        assert!(!single.vt_only());

        let vt_only = Snapshot {
            vt: true,
            ..Snapshot::default()
        };
        assert_eq!(vt_only.channel_state(), ChannelState::None);
        assert!(!vt_only.any_channel_active());
        assert!(vt_only.vt_only());

        let multi = Snapshot {
            d0: true,
            d1: false,
            d2: true,
            d3: false,
            vt: true,
        };
        assert_eq!(multi.channel_bits(), D0_BIT | D2_BIT);
        assert_eq!(multi.active_channel(), None);
        assert_eq!(
            multi.channel_state(),
            ChannelState::Multiple(D0_BIT | D2_BIT)
        );
    }

    #[test]
    fn event_helpers_report_changed_mask_and_edges() {
        let event = Event {
            previous: Snapshot::default(),
            current: Snapshot {
                d0: true,
                vt: true,
                ..Snapshot::default()
            },
        };

        assert_eq!(event.changed_mask(), D0_BIT | VT_BIT);
        assert_eq!(event.edge(Signal::D0), Some(Edge::Rising));
        assert_eq!(event.edge(Signal::VT), Some(Edge::Rising));
        assert!(event.vt_rising());
        assert!(!event.vt_falling());

        let falling = Event {
            previous: event.current,
            current: Snapshot::default(),
        };
        assert_eq!(falling.edge(Signal::D0), Some(Edge::Falling));
        assert_eq!(falling.edge(Signal::VT), Some(Edge::Falling));
        assert!(falling.vt_falling());
    }

    #[test]
    fn poll_change_suppresses_repeated_same_state() {
        let mut dev = Rx480eWq::new(
            MockPin::new(false),
            MockPin::new(false),
            MockPin::new(false),
            MockPin::new(false),
            MockPin::new(false),
        );

        assert!(dev.poll_change().unwrap().is_none());
        assert!(dev.poll_change().unwrap().is_none());
    }

    #[test]
    fn channel_state_from_channel_bits_detects_multi_channel() {
        assert_eq!(channel_state_from_channel_bits(0), ChannelState::None);
        assert_eq!(
            channel_state_from_channel_bits(D3_BIT),
            ChannelState::Single(Channel::D3)
        );
        assert_eq!(
            channel_state_from_channel_bits(D0_BIT | D1_BIT),
            ChannelState::Multiple(D0_BIT | D1_BIT)
        );
        assert_eq!(channel_state_from_channel_bits(VT_BIT), ChannelState::None);
    }

    #[test]
    fn sample_propagates_pin_errors() {
        let mut dev = Rx480eWq::new(ErrorPin, ErrorPin, ErrorPin, ErrorPin, ErrorPin);
        assert_eq!(dev.sample(), Err(PinError));
    }
}
