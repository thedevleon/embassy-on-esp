# Embassy on ESP

## Setup
See https://esp-rs.github.io/book/installation/index.html.
-> you'll only need the RISC-V targets. This project is no_std, so you also don't need to follow the `std Development Requirements`.

## How to run?
Attach your ESP32 via the USB port that is hooked up to the ESP directly (not the UART port), and run `cargo run`.
On initial flash you'll need to put the ESP into bootloader mode by holding the GPIO 9 button and resseting the ESP (via the EN/reset button).