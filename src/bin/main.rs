#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use esp_hal::analog::adc::{AdcConfig, Adc, Attenuation};
use esp_hal::clock::CpuClock;
use panic_rtt_target as _;
use oled_i2c::{Oled, OledConfig, OledKind};

use esp_hal::{
    delay::Delay,
    i2c::master::{Config, I2c},
    time::Rate,
};
use embedded_graphics::{
    primitives::*,
    draw_target::*,
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    geometry::{Point, Size},
    text::Text,
    Drawable,
    mono_font::iso_8859_7::{
        FONT_4X6 as FONT_SUB,
        FONT_6X10 as FONT_MAIN,
    }
};
use panic_rtt_target as _;

esp_bootloader_esp_idf::esp_app_desc!();

fn draw_text<D>(display: &mut D, x: i32, y: i32, text: &str) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let main = MonoTextStyle::new(&FONT_MAIN, BinaryColor::On);
    let sub = MonoTextStyle::new(&FONT_SUB, BinaryColor::On);

    let mut cursor = Point::new(x, y);
    let mut rest = text;

    while let Some(c) = rest.chars().next() {
        match rest.strip_prefix("_{") {
            Some(after) if let Some((inside, remaining)) = after.split_once('}') => {
                let next = Text::new(inside, cursor + Point::new(0, 2), sub)
                    .draw(display)?;
                cursor.x = next.x + 1;
                rest = remaining;
                continue;
            }
            _ => (),
        }

        let (this, remaining) = rest.split_at(c.len_utf8());

        if c == '∠' {
            Line::new(cursor, cursor + Point::new(6, 0))
                .into_styled(stroke)
                .draw(display)?;
            Line::new(cursor, cursor + Point::new(5, -6))
                .into_styled(stroke)
                .draw(display)?;
            cursor.x += 8;
        } else {
            cursor = Text::new(this, cursor, main).draw(display)?;
        }

        rest = remaining;
    }

    Ok(())
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Potentiometer
    let analog_pin = peripherals.GPIO3;
    let mut adc1_config = AdcConfig::new();
    let mut pin = adc1_config.enable_pin(analog_pin, Attenuation::_11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    // Display
    let i2c = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(Rate::from_khz(400)),
    ).expect("I2C config failed")
     .with_sda(peripherals.GPIO8)
     .with_scl(peripherals.GPIO9);

    let mut display = Oled::new(
        i2c,
        0x3C,
        OledConfig::new(OledKind::Sh1106, 128, 64).with_contrast(0xCF),
    ).expect("OLED init failed");

    let delay = Delay::new();

    display.clear().expect("clear failed");
    draw_text(&mut display, 11, 12, "Ackermann Steering").unwrap();
    draw_text(&mut display, 42, 28, "δ_{i} = ∠ 8°").unwrap();
    draw_text(&mut display, 42, 44, "δ_{o} = ∠12°").unwrap();

    let mut offset = 0;

    loop {
        let pin_value: u16 = nb::block!(adc1.read_oneshot(&mut pin)).unwrap();
        info!("{}", pin_value);

        // Clear last line
        display.fill_solid(&Rectangle::new(Point::new(0, 52), Size::new(128, 16)),BinaryColor::Off).unwrap();

        // Draw updated text
        draw_text(&mut display, offset+300, 60, "Ahmed Salah Anas Mabrouk Yusuf Ghazi Zeyad Tarek").unwrap();
        draw_text(&mut display, offset, 60, "Ahmed Salah Anas Mabrouk Yusuf Ghazi Zeyad Tarek").unwrap();

        display.flush().expect("flush failed");

        if offset < -298 {
            offset = 0;
        }
        else {
            offset = offset.wrapping_sub(1);
        }

        delay.delay_millis(16);
    }
}
