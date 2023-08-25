use esp_idf_sys::esp_random;
use smart_leds::hsv::hsv2rgb;
use smart_leds::hsv::Hsv;
use smart_leds::{SmartLedsWrite};
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::driver::color::LedPixelColorGrbw32;
use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, RGB8};

const LED_PIN: u32 = 21; // 2: M5Stamp S3
const NUM_PIXELS: usize = 1;

fn main() -> ! {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut ws2812 = LedPixelEsp32Rmt::<RGB8, LedPixelColorGrbw32>::new(0, LED_PIN).unwrap();

    loop {
        let mut hue = unsafe { esp_random() } as u8;
        loop {
            // NUM_PIXEL 分、同じ色の iterator をつくって write() に渡す
            let pixels = std::iter::repeat(hsv2rgb(Hsv {
                hue,
                sat: 255,
                val: 8,
            }))
            .take(NUM_PIXELS);
            ws2812.write(pixels).unwrap();

            sleep(Duration::from_millis(5));

            hue = hue.wrapping_add(1);
        }
    }
}