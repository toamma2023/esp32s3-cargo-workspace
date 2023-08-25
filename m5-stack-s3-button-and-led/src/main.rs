use std::thread;
use std::sync::{Mutex, Arc};

use esp_idf_hal::{gpio::*, prelude::*, task};
use esp_idf_sys::{tskTaskControlBlock, TaskHandle_t};
use log::*;
use std::sync::atomic::{AtomicPtr, Ordering};
use libc::{c_void};

use esp_idf_sys::esp_random;
use smart_leds::hsv::hsv2rgb;
use smart_leds::hsv::Hsv;
use smart_leds::{SmartLedsWrite};
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::driver::color::LedPixelColorGrbw32;
use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, RGB8};

const LED_PIN: u32 = 21; // 2: M5Stamp SC
const NUM_PIXELS: usize = 1;

fn main() -> ! {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    const BUTTON_NOTIFICATION: u32 = 1;

    let task_handle: AtomicPtr<tskTaskControlBlock> = AtomicPtr::new(std::ptr::null_mut());
    let ptr: TaskHandle_t = task::current().expect("never fail.");
    task_handle.store(ptr as *mut tskTaskControlBlock, Ordering::Relaxed);

    let peripherals = Peripherals::take().expect("never fail");
    let button_pin = peripherals.pins.gpio0;

    let mut button = PinDriver::input(button_pin).expect("never fail");
    let _ = button.set_pull(Pull::Up);
    let _ = button.set_interrupt_type(InterruptType::NegEdge);

    let share_flag = Arc::new(Mutex::new(0));

    println!("button ready");
    unsafe {
        let _ = button.subscribe(move || {
            task::notify(task_handle.load(Ordering::Relaxed) as *mut c_void, BUTTON_NOTIFICATION);
            //println!("button pressed");
        });
    }
    
    let share_flag_1 = Arc::clone(&share_flag);
    thread::spawn(move || {
        let mut ws2812 = LedPixelEsp32Rmt::<RGB8, LedPixelColorGrbw32>::new(0, LED_PIN).unwrap();

        let mut hue = unsafe { esp_random() } as u8;
        loop {
            let flag = share_flag_1.lock().unwrap();
            let flg = *flag; 
            drop(flag);
            if flg % 2 == 0 {
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

            } else {
                let pixels = std::iter::repeat(RGB8::from((6, 0, 0))).take(1);
                ws2812.write(pixels).unwrap();
                sleep(Duration::from_millis(250));
        
                let pixels = std::iter::repeat(RGB8::from((0, 6, 0))).take(1);
                ws2812.write(pixels).unwrap();
                sleep(Duration::from_millis(250));
        
                let pixels = std::iter::repeat(RGB8::from((0, 0, 6))).take(1);
                ws2812.write(pixels).unwrap();
                sleep(Duration::from_millis(250));
        
                let pixels = std::iter::repeat(RGB8::from((0, 0, 0))).take(1);
                ws2812.write(pixels).unwrap();
                sleep(Duration::from_millis(250));
        
            }
        }
    });

    loop {
        let res = task::wait_notification(Some(Duration::from_secs(1)));
        if let Some(BUTTON_NOTIFICATION) = res {
            info!("button pressed");
            let mut flag = share_flag.lock().unwrap();
            *flag = *flag + 1;
        }
    }
}