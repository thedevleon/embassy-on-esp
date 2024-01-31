# Embassy on ESP
with no_std, async, embassy, leds and more!

## Hardware
ESP32-C6-DevKitM-1 (for now, until a proper C6 devboard comes around... or maybe I should just make one?)

## Setup
1. Install Rust >=1.75 via rustup (https://rustup.rs/)
2. `rustup target add riscv32imac-unknown-none-elf`


## How to run?
Attach the ESP32 via the USB port that is hooked up to the ESP directly (not the UART port), and run `cargo run`.

On initial flash you'll need to put the ESP into bootloader mode by holding the GPIO 9 button and resseting the ESP (via the EN/reset button).