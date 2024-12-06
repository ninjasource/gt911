# GT911 Touchscreen Driver

A Rust driver for the Goodix GT911 touch screen device

Supports both blocking and async modes of operation and up to 5 touch points. 
The GT911 supports triggering an interrupt for touch events but this driver has not yet implemented that functionality.
Therefore the examples below are for polling the state of the device (usually done for every rendered frame).
The driver is stateless so it is up to the user to keep track of touch points in order to figure out what is pressed and released. 
See full example at the end.

# Examples

## Single-touch async poll example

```rust
    let touch = Gt911::default();
    touch.init(&mut i2c).await.unwrap();

    if let Ok(point) = touch.get_touch(&mut i2c).await {
        // point can be Some (pressed or moved) or None (released)
    } else {
        // ignore because nothing has happened since last poll => Error::NotReady
    }

```

## Muiti-touch async poll example 


```rust

    let touch = Gt911::default();
    touch.init(&mut i2c).await.unwrap();

    if let Ok(points) = touch.get_multi_touch(&mut i2c).await {
        // stack allocated Vec containing 0-5 points
    }
```



## Single-touch blocking poll example

```rust
    let touch = Gt911Blocking::default();
    touch.init(&mut i2c).unwrap();

    if let Ok(point) = touch.get_touch(&mut i2c) {
        // point can be Some (pressed or moved) or None (released)
    }

```

## Muiti-touch blocking exapoll examplemple 


```rust

    let touch = Gt911Blocking::default();
    touch.init(&mut i2c).unwrap();

    if let Ok(points) = touch.get_multi_touch(&mut i2c) {
        // stack allocated Vec containing 0-5 points
    }
```



## Full single touch async poll example using Embassy and an stm32u5g9j-dk2

```rust
#![no_std]
#![no_main]
#![macro_use]

use {
    defmt::info,
    defmt_rtt as _,
    gt911::Gt911,
    embassy_executor::Spawner,
    embassy_stm32::{
        bind_interrupts,
        i2c::{self, I2c},
        peripherals,
        time::Hertz,
    },
    panic_probe as _,
};

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

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

    // connect with the default i2c address
    let touch = Gt911::default();
    touch.init(&mut i2c).await.unwrap();

    let mut last_point = None;

    loop {
        // get single touchpoint
        if let Ok(point) = touch.get_touch(&mut i2c).await {
            match point {
                Some(point) => match last_point.replace(point.clone()) {
                    Some(old_point) => {
                        if point != old_point {
                            info!("moved: {:?}", point);
                        }
                    }
                    None => {
                        info!("pressed: {:?}", point);
                    }
                },
                None => {
                    let point = last_point.take();
                    info!("released: {:?}", point);
                }
            };
        }
    }
}
```