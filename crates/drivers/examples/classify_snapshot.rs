use rx480e_wq_driver::{Channel, ChannelState, Snapshot};

fn main() {
    let snapshot = Snapshot {
        d0: true,
        d1: false,
        d2: false,
        d3: false,
        vt: true,
    };

    assert_eq!(snapshot.channel_bits(), Channel::D0.bit());
    assert!(snapshot.any_channel_active());
    assert!(snapshot.is_valid_transmission());
    assert_eq!(snapshot.active_channel(), Some(Channel::D0));
    assert_eq!(snapshot.channel_state(), ChannelState::Single(Channel::D0));
    assert!(!snapshot.vt_only());

    if let ChannelState::Single(channel) = snapshot.channel_state() {
        println!("{} active", channel.name());
    }
}
