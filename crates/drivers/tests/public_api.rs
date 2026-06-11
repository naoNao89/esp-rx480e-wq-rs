use std::{cell::Cell, rc::Rc};

use embedded_hal::digital::InputPin;
use rx480e_wq_driver::{
    channel_state_from_bits, Channel, ChannelState, Edge, Event, Rx480eWq, Snapshot, D0_BIT,
    D1_BIT, D2_BIT, D3_BIT, VT_BIT,
};

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

#[test]
fn snapshot_api_returns_structured_state_not_debug_text() {
    let snapshot = Snapshot {
        d0: true,
        d1: false,
        d2: false,
        d3: false,
        vt: true,
    };

    assert_eq!(snapshot.channel_bits(), D0_BIT);
    assert_eq!(snapshot.bits(), D0_BIT | VT_BIT);
    assert_eq!(snapshot.active_channel(), Some(Channel::D0));
    assert_eq!(snapshot.channel_state(), ChannelState::Single(Channel::D0));
    assert!(!snapshot.vt_only());
}

#[test]
fn vt_only_means_vt_active_without_d0_to_d3() {
    let snapshot = Snapshot {
        d0: false,
        d1: false,
        d2: false,
        d3: false,
        vt: true,
    };

    assert_eq!(snapshot.channel_bits(), 0);
    assert_eq!(snapshot.channel_state(), ChannelState::None);
    assert!(snapshot.vt_only());
}

#[test]
fn multi_channel_is_structured_mask() {
    let snapshot = Snapshot {
        d0: false,
        d1: true,
        d2: true,
        d3: false,
        vt: true,
    };

    assert_eq!(snapshot.active_channel(), None);
    assert_eq!(snapshot.channel_bits(), D1_BIT | D2_BIT);
    assert_eq!(
        snapshot.channel_state(),
        ChannelState::Multiple(D1_BIT | D2_BIT)
    );
}

#[test]
fn event_api_returns_edges_and_changed_mask() {
    let event = Event {
        previous: Snapshot::default(),
        current: Snapshot {
            d3: true,
            vt: true,
            ..Snapshot::default()
        },
    };

    assert_eq!(event.changed_mask(), D3_BIT | VT_BIT);
    assert_eq!(event.edge(Channel::D3), Some(Edge::Rising));
    assert_eq!(event.edge(Channel::VT), Some(Edge::Rising));
    assert!(event.vt_rising());
    assert!(!event.vt_falling());
}

#[test]
fn poll_change_produces_events_not_printed_debug_lines() {
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
    let event = rx.poll_change().unwrap().expect("structured event");

    assert!(event.vt_rising());
    assert_eq!(
        event.current.channel_state(),
        ChannelState::Single(Channel::D0)
    );
}

#[test]
fn channel_state_from_bits_matches_all_single_channels() {
    assert_eq!(
        channel_state_from_bits(D0_BIT),
        ChannelState::Single(Channel::D0)
    );
    assert_eq!(
        channel_state_from_bits(D1_BIT),
        ChannelState::Single(Channel::D1)
    );
    assert_eq!(
        channel_state_from_bits(D2_BIT),
        ChannelState::Single(Channel::D2)
    );
    assert_eq!(
        channel_state_from_bits(D3_BIT),
        ChannelState::Single(Channel::D3)
    );
    assert_eq!(channel_state_from_bits(0), ChannelState::None);
    assert_eq!(
        channel_state_from_bits(D0_BIT | D3_BIT),
        ChannelState::Multiple(D0_BIT | D3_BIT)
    );
}
