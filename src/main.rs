#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, Ticker, Instant};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp32c6_hal::{clock::ClockControl, embassy, peripherals::Peripherals, prelude::*, rmt::Rmt, IO};
use esp_backtrace as _;
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{
    brightness, gamma, SmartLedsWrite, RGB8
};
use rgb::RGBA8;

enum LedAnimation {
    SolidColor(RGBA8), // color
    FadeOnOff(RGBA8, u64), // color, duration
    FadeFromTo(RGBA8, RGBA8, u64), // color1, color2, duration 
    Blink(RGBA8, u64, u64), // color, duration, interval,
    BlinkBurst(RGBA8, u8, u64, u64), // color, bursts, duration, interval
    Off,
}

// LED Animation Queue as https://docs.embassy.dev/embassy-sync/git/default/signal/struct.Signal.html
static LED_ANIMATION_SIGNAL: Signal<CriticalSectionRawMutex, LedAnimation> = Signal::new();

#[embassy_executor::task] // Note: embassy does not yet support generics in tasks
async fn led_animator(mut led: SmartLedsAdapter<esp32c6_hal::rmt::Channel<0>, 0, 25>) {
    let ticker_duration = Duration::from_millis(33); // 30 Hz
    let mut ticker = Ticker::every(ticker_duration.clone());
    let mut start_time: Instant = Instant::now();
    let mut current_led_animation = LedAnimation::SolidColor(RGBA8{r: 0, g: 0, b: 0, a: 0});
    let mut current_color = RGB8{r: 0, g: 0, b: 0};
    let mut current_brightness: u8 = 0;

    loop {
        // get the new animation from the queue if one is available
        if LED_ANIMATION_SIGNAL.signaled()
        {
            current_led_animation = LED_ANIMATION_SIGNAL.wait().await;
        }

        let now = Instant::now();
        
        match current_led_animation {
            LedAnimation::SolidColor(color) => {
                current_color = RGB8{r: color.r, g: color.g, b: color.b};
                current_brightness = color.a;
    
            },
            LedAnimation::FadeOnOff(color, duration) => {
                let mut progress = (now - start_time).as_millis() as f32 / duration as f32;

                // wrap around
                if progress > 1.0 {
                    start_time = now;
                    progress = progress - 1.0;
                }

                current_color = RGB8{r: color.r, g: color.g, b: color.b};

                if progress < 0.5 {
                    // Fade On
                    current_brightness = (color.a as f32 * (progress * 2.0)) as u8; 
                } else if progress > 0.5 {
                    // Fade Off
                    current_brightness = (color.a as f32 * (1.0 - ((progress - 0.5) * 2.0))) as u8;
                }
            },
            LedAnimation::FadeFromTo(color1, color2, duration) => {
                // TODO: fade from color1 to color2 over the duration
            },
            LedAnimation::Blink(color, duration, interval) => {
                // TODO: blink the color over the duration with the interval
            },
            LedAnimation::BlinkBurst(color, bursts, duration, interval) => {
                // TODO: blink the color over the duration with the interval
            },
            LedAnimation::Off => {
                // TODO: turn off the led
            },
        }

        //log::info!("color: {:?}, brightness: {}", current_color, current_brightness);

        led.write(gamma(brightness([current_color].iter().cloned(), current_brightness))).unwrap();

        ticker.next().await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();

    log::info!("Hello, world!");

    // This needs the features esp32c6-hal/embassy-time-systick !AND! embassy-time/tick-hz-16_000_000
    embassy::init(
        &clocks,
        esp32c6_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    let rmt = Rmt::new(peripherals.RMT, 80u32.MHz(), &clocks).unwrap();
    let rmt_buffer = smartLedBuffer!(1);
    let led = SmartLedsAdapter::new(rmt.channel0, io.pins.gpio8, rmt_buffer);

    spawner.spawn(led_animator(led)).ok();

    // Try all the different animations
    loop {

        // purple and blue
        let color1 = RGBA8{r: 138, g: 43, b: 226, a: 255};
        let color2 = RGBA8{r: 0, g: 0, b: 255, a: 255};

        log::info!("Testing: RGB");
        LED_ANIMATION_SIGNAL.signal(LedAnimation::SolidColor(RGBA8{r: 255, g: 0, b: 0, a: 128}));
        Timer::after(Duration::from_millis(2_000)).await;
        LED_ANIMATION_SIGNAL.signal(LedAnimation::SolidColor(RGBA8{r: 0, g: 255, b: 0, a: 128}));
        Timer::after(Duration::from_millis(2_000)).await;
        LED_ANIMATION_SIGNAL.signal(LedAnimation::SolidColor(RGBA8{r: 0, g: 0, b: 155, a: 128}));
        Timer::after(Duration::from_millis(2_000)).await;

        // SolidColor
        log::info!("Testing: SolidColor");
        LED_ANIMATION_SIGNAL.signal(LedAnimation::SolidColor(color1));
        Timer::after(Duration::from_millis(10_000)).await;

        // FadeOnOff
        log::info!("Testing: FadeOnOff");
        LED_ANIMATION_SIGNAL.signal(LedAnimation::FadeOnOff(color1, 2_000));
        Timer::after(Duration::from_millis(10_000)).await;

        // FadeFromTo
        log::info!("Testing: FadeFromTo");
        LED_ANIMATION_SIGNAL.signal(LedAnimation::FadeFromTo(color1, color2, 3_000));
        Timer::after(Duration::from_millis(10_000)).await;

        // Blink
        log::info!("Testing: Blink");
        LED_ANIMATION_SIGNAL.signal(LedAnimation::Blink(color1, 500, 1_000));
        Timer::after(Duration::from_millis(10_000)).await;

        // BlinkBurst
        log::info!("Testing: BlinkBurst");
        LED_ANIMATION_SIGNAL.signal(LedAnimation::BlinkBurst(color1, 3, 100, 1_000));
        Timer::after(Duration::from_millis(10_000)).await;

        // Off
        log::info!("Testing: Off");
        LED_ANIMATION_SIGNAL.signal(LedAnimation::Off);
        Timer::after(Duration::from_millis(10_000)).await;
    }
}