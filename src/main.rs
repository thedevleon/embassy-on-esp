#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp32c6_hal::{clock::ClockControl, embassy, peripherals::Peripherals, prelude::*, rmt::Rmt, IO, gpio::GpioPin, gpio::Output, gpio::PushPull};
use esp_backtrace as _;
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{
    brightness, gamma, SmartLedsWrite, RGB8
};
use rgb::RGBA8;

enum LedAnimation {
    SolidColor(RGBA8), // color
    FadeOnOff(RGBA8, f32), // color, duration
    FadeFromTo(RGBA8, RGBA8, f32), // color1, color2, duration 
    Blink(RGBA8, f32, f32), // color, duration, interval,
    BlinkBurst(RGBA8, u8, f32, f32), // color, bursts, duration, interval
    Off,
}

struct AnimationState {
    // TODO add state
    new_color: RGB8,
    new_brightness: u8,
}

// LED Animation Queue as https://docs.embassy.dev/embassy-sync/git/default/signal/struct.Signal.html
static LED_ANIMATION_SIGNAL: Signal<CriticalSectionRawMutex, LedAnimation> = Signal::new();

#[embassy_executor::task]
async fn led_animator(mut led: SmartLedsAdapter<esp32c6_hal::rmt::Channel<0>, 0, 25>) { // TODO types of channel and gpio
    let mut state: AnimationState = AnimationState {
        new_color: RGB8{r: 0, g: 0, b: 0},
        new_brightness: 0,
    };
    let mut current_led_animation = LedAnimation::SolidColor(RGBA8{r: 0, g: 0, b: 0, a: 0});

    loop {
        // get the new animation from the queue if one is available
        if LED_ANIMATION_SIGNAL.signaled()
        {
            current_led_animation = LED_ANIMATION_SIGNAL.wait().await;
        }
        
        match current_led_animation {
            LedAnimation::SolidColor(color) => {
                // TODO: set the color
                //state.new_color = color;
                state.new_color = RGB8{r: color.r, g: color.g, b: color.b};
                state.new_brightness = color.a;
    
            },
            LedAnimation::FadeOnOff(color, duration) => {
                // TODO: fade to the color over the duration
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

        led.write(gamma(brightness([state.new_color].iter().cloned(), state.new_brightness))).unwrap();

        // 30 Hz
        Timer::after(Duration::from_millis(33)).await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // setup logger
    // To change the log_level change the env section in .cargo/config.toml
    // or remove it and set ESP_LOGLEVEL manually before running cargo run
    // this requires a clean rebuild because of https://github.com/rust-lang/cargo/issues/10358
    esp_println::logger::init_logger_from_env();

    // something feels odd with the clock setup... it's running a lot faster than it should
    embassy::init(
        &clocks,
        esp32c6_hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );

    let rmt = Rmt::new(peripherals.RMT, 80u32.MHz(), &clocks).unwrap();
    let rmt_buffer = smartLedBuffer!(1);
    let mut led = SmartLedsAdapter::new(rmt.channel0, io.pins.gpio8, rmt_buffer);

    spawner.spawn(led_animator(led)).ok();
    // TODO: replace once we know the correct types for the channel and gpio
    // spawner.spawn(led_animator(rmt.channel0, io.pins.gpio8)).ok();

    loop {
        // Try all the different animations

        // purple and blue
        let color1 = RGBA8{r: 138, g: 43, b: 226, a: 128};
        let color2 = RGBA8{r: 0, g: 0, b: 255, a: 128};

        LED_ANIMATION_SIGNAL.signal(LedAnimation::SolidColor(RGBA8{r: 255, g: 0, b: 0, a: 128}));
        Timer::after(Duration::from_millis(50_000)).await;
        LED_ANIMATION_SIGNAL.signal(LedAnimation::SolidColor(RGBA8{r: 0, g: 255, b: 0, a: 128}));
        Timer::after(Duration::from_millis(50_000)).await;
        LED_ANIMATION_SIGNAL.signal(LedAnimation::SolidColor(RGBA8{r: 0, g: 255, b: 155, a: 128}));
        Timer::after(Duration::from_millis(50_000)).await;

        // SolidColor
        LED_ANIMATION_SIGNAL.signal(LedAnimation::SolidColor(color1));
        Timer::after(Duration::from_millis(100_000)).await;

        // FadeOnOff
        LED_ANIMATION_SIGNAL.signal(LedAnimation::FadeOnOff(color1, 2.0));
        Timer::after(Duration::from_millis(100_000)).await;

        // FadeFromTo
        LED_ANIMATION_SIGNAL.signal(LedAnimation::FadeFromTo(color1, color2, 3.0));
        Timer::after(Duration::from_millis(100_000)).await;

        // Blink
        LED_ANIMATION_SIGNAL.signal(LedAnimation::Blink(color1, 0.5, 1.0));
        Timer::after(Duration::from_millis(100_000)).await;

        // BlinkBurst
        LED_ANIMATION_SIGNAL.signal(LedAnimation::BlinkBurst(color1, 3, 0.1, 1.0));
        Timer::after(Duration::from_millis(100_000)).await;

        // Off
        LED_ANIMATION_SIGNAL.signal(LedAnimation::Off);
        Timer::after(Duration::from_millis(100_000)).await;
    }
}