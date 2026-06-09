#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Pull},
    main,
};
use esp_println::println;

const SAMPLE_PERIOD_MS: u32 = 5;

fn channel_bits(snapshot: &rx480e_wq_driver::Snapshot) -> u8 {
    (snapshot.d0 as u8)
        | ((snapshot.d1 as u8) << 1)
        | ((snapshot.d2 as u8) << 2)
        | ((snapshot.d3 as u8) << 3)
}

fn key_from_bits(bits: u8) -> Option<&'static str> {
    match bits {
        0b0001 => Some("D0"),
        0b0010 => Some("D1"),
        0b0100 => Some("D2"),
        0b1000 => Some("D3"),
        _ => None,
    }
}

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
            let bits = channel_bits(&event.current);

            if !event.previous.vt && event.current.vt {
                active_start_ms = ms;
                seen_channel_bits = bits;
            }

            if event.current.vt {
                seen_channel_bits |= bits;
            }

            if event.previous.vt && !event.current.vt {
                let pulse_ms = ms.wrapping_sub(active_start_ms);
                match key_from_bits(seen_channel_bits) {
                    Some(key) => println!("EVENT: key={} vt=1 pulse_ms={}", key, pulse_ms),
                    None if seen_channel_bits == 0 => {
                        println!("EVENT: vt_only pulse_ms={}", pulse_ms)
                    }
                    None => println!(
                        "EVENT: multi_channel mask=0x{:x} vt=1 pulse_ms={}",
                        seen_channel_bits, pulse_ms
                    ),
                }
                seen_channel_bits = 0;
            }
        }

        ms = ms.wrapping_add(SAMPLE_PERIOD_MS);
        delay.delay_millis(SAMPLE_PERIOD_MS);
    }
}
