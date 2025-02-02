#![no_std]
#![no_main]
#![macro_use]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::{i2c::I2c, time::Hertz};
use embassy_time::{Duration, Timer};
use gt911::{Error, Gt911Blocking};
use stm32u5_examples::rcc_setup;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = rcc_setup::stm32u5g9zj_init();

    let mut i2c = I2c::new_blocking(p.I2C2, p.PF1, p.PF0, Hertz(100_000), Default::default());

    let touch = Gt911Blocking::default();
    touch.init(&mut i2c).unwrap();

    loop {
        match touch.get_touch(&mut i2c) {
            Ok(point) => {
                info!("{:?}", point);
            }
            Err(Error::NotReady) => {
                // ignore, the touchscreen has no new data for us
            }
            Err(e) => error!("Error: {:?}", e),
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
