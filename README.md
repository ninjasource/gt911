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

```

## Muiti-touch async poll example 


```rust

    let touch = Gt911::default();
    let mut buf = [0u8; gt911::GET_MULTITOUCH_BUF_SIZE];

    touch.init(&mut i2c, &mut buf).await.unwrap();

    loop {
        if let Ok(points) = touch.get_multi_touch(&mut i2c, &mut buf).await {
            // stack allocated Vec containing 0-5 points
            info!("{:?}", points)
        }
    }
```



## Single-touch blocking poll example

```rust
    let touch = Gt911Blocking::default();
    touch.init(&mut i2c).unwrap();

    loop {
        if let Ok(point) = touch.get_touch(&mut i2c) {
            // point can be Some (pressed or moved) or None (released)
            info!("{:?}", point)
        } else {
            // ignore because nothing has happened since last poll => Error::NotReady
        }
    }

```

## Muiti-touch blocking poll example


```rust
    let touch = Gt911Blocking::default();
    touch.init(&mut i2c).unwrap();

    loop {
        if let Ok(points) = touch.get_multi_touch(&mut i2c) {
            // stack allocated Vec containing 0-5 points
            info!("{:?}", points)
        }
    }
```

See Examples folder for full examples

# Why the async version is different

Why does the async version take a read buffer and not the blocking version? 
Some mcu's support DCACHE but have data cache coherency issues with DMA (stm32h7 mcus in particular). 
In order to address this the user can exclude a special memory region from DCACHE and use this buffer for i2c communication over await points. Alternatively the user can disable dcache.
