#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Pull},
    main,
};
use esp_println::println;
use rx480e_wq_driver::{ChannelState, channel_state_from_channel_bits};

const SAMPLE_PERIOD_MS: u32 = 5;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    // RX480-E-WQ wiring currently under test:
    // D0 -> GPIO0, D1 -> GPIO1, D2 -> GPIO3, D3 -> GPIO4, VT -> GPIO5.
    // Use weak pull-downs so idle/disconnected inputs read low, while RX480
    // CMOS outputs can still drive them high.
    let input_config = InputConfig::default().with_pull(Pull::Down);
    let d0 = Input::new(peripherals.GPIO0, input_config);
    let d1 = Input::new(peripherals.GPIO1, input_config);
    let d2 = Input::new(peripherals.GPIO3, input_config);
    let d3 = Input::new(peripherals.GPIO4, input_config);
    let vt = Input::new(peripherals.GPIO5, input_config);
    let mut rx = rx480e_wq_driver::Rx480eWq::new(d0, d1, d2, d3, vt);

    println!("RX480-E-WQ ESP32-C3 reader started");
    println!("Wiring: D0=GPIO0 D1=GPIO1 D2=GPIO3 D3=GPIO4 VT=GPIO5");
    println!("Press the remote after learning/pairing the RX480 module.");
    println!("Note: the module may already contain learned codes from factory/previous use.");

    let mut active_start_ms: u32 = 0;
    let mut seen_channel_bits: u8 = 0;
    let mut ms: u32 = 0;

    loop {
        if let Ok(Some(event)) = rx.poll_change() {
            let bits = event.current.channel_bits();

            if event.vt_rising() {
                active_start_ms = ms;
                seen_channel_bits = bits;
            }

            if event.current.vt {
                seen_channel_bits |= bits;
            }

            if event.vt_falling() {
                let pulse_ms = ms.wrapping_sub(active_start_ms);
                match channel_state_from_channel_bits(seen_channel_bits) {
                    ChannelState::Single(channel) => {
                        println!("EVENT: key={} vt=1 pulse_ms={}", channel.name(), pulse_ms)
                    }
                    ChannelState::None => {
                        println!("EVENT: vt_only pulse_ms={}", pulse_ms)
                    }
                    ChannelState::Multiple(mask) => println!(
                        "EVENT: multi_channel mask=0x{:x} vt=1 pulse_ms={}",
                        mask, pulse_ms
                    ),
                }
                seen_channel_bits = 0;
            }
        }

        ms = ms.wrapping_add(SAMPLE_PERIOD_MS);
        delay.delay_millis(SAMPLE_PERIOD_MS);
    }
}
