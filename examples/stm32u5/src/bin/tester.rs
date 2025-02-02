#![no_std]
#![no_main]
#![macro_use]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    i2c::{self, I2c},
    peripherals,
    time::Hertz,
};
use embassy_time::{Duration, Timer};
use gt911::{Error, Gt911};
use stm32u5_examples::rcc_setup;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = rcc_setup::stm32u5g9zj_init();

    // used for the touch events
    let mut i2c = I2c::new(
        p.I2C2,
        p.PF1,
        p.PF0,
        Irqs,
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
        Hertz(100_000),
        Default::default(),
    );

    let touch = Gt911::default();
    let mut buf = [0u8; gt911::GET_TOUCH_BUF_SIZE];

    touch.init(&mut i2c, &mut buf).await.unwrap();

    loop {
        if let Ok(point) = touch.get_touch(&mut i2c, &mut buf).await {
            // point can be Some (pressed or moved) or None (released)
            info!("{:?}", point)
        } else {
            // ignore because nothing has happened since last poll => Error::NotReady
        }
    }
}
