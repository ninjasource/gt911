// A basic driver for the Goodix GT911 touch screen
// This implementation supports both blocking and async communication over I2C

#![no_std]

use core::{marker::PhantomData, str};

const GT911_I2C_ADDR_BA: u8 = 0x5D;
const GT911_PRODUCT_ID_REG: u16 = 0x8140;
const GT911_TOUCHPOINT_STATUS_REG: u16 = 0x814E;
const GT911_TOUCHPOINT_1_REG: u16 = 0x814F;
const GT911_COMMAND_REG: u16 = 0x8040;

const MAX_NUM_TOUCHPOINTS: usize = 5;
const TOUCHPOINT_ENTRY_LEN: usize = 8;

/// The touchpoint
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point {
    /// The touchpoint number (zero based)
    pub track_id: u8,
    /// x coordinate in screen pixels
    pub x: u16,
    /// y coordinate in screen pixels
    pub y: u16,
    /// How much area the finder takes up on the touch point
    pub area: u16,
}

/// Gt911 Error
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub enum Error<E> {
    /// Usually indicates that you are attempting to communicate with a device that is not a 911
    /// or that there is a general communication failure
    UnexpectedProductId,
    /// I2C communication error
    I2C(E),
    /// Not an actual error, it just means "no new data available"
    /// This means that you have polled the device again in-between it detecting any new touch data
    /// This can safely be ignored
    NotReady,
}

/// Blocking Gt911
pub struct Gt911Blocking<I2C> {
    i2c_addr: u8, // e.g. 0x5D
    i2c: PhantomData<I2C>,
}

/// Use the default I2C address for communication
impl<I2C> Default for Gt911Blocking<I2C> {
    fn default() -> Self {
        Self {
            i2c_addr: GT911_I2C_ADDR_BA,
            i2c: PhantomData,
        }
    }
}

/// Blocking Gt911 implementation
impl<I2C, E> Gt911Blocking<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    /// Creates a new instance with a user specified i2c address
    pub fn new(i2c_addr: u8) -> Self {
        Self {
            i2c_addr,
            i2c: PhantomData,
        }
    }

    /// Checks the ProductId for a "911\0" string response and resets the status register
    /// Only needs to be called once on startup
    pub fn init(&self, i2c: &mut I2C) -> Result<(), Error<E>> {
        // switch to command mode
        self.write(i2c, GT911_COMMAND_REG, 0)?;

        // read the product_id and confirm that it is expected
        let mut read = [0u8; 4];
        self.read(i2c, GT911_PRODUCT_ID_REG, &mut read)?;
        match str::from_utf8(&read) {
            Ok(product_id) => {
                if product_id != "911\0" {
                    return Err(Error::UnexpectedProductId);
                }
            }
            Err(_) => {
                return Err(Error::UnexpectedProductId);
            }
        }

        // clear status register
        self.write(i2c, GT911_TOUCHPOINT_STATUS_REG, 0)?;
        Ok(())
    }

    /// Gets a single touch point
    /// Returns Ok(None) for release, Some(point) for press or move and Err(Error::NotReady) for no data
    pub fn get_touch(&self, i2c: &mut I2C) -> Result<Option<Point>, Error<E>> {
        let num_touch_points = self.get_num_touch_points(i2c)?;

        let point = if num_touch_points > 0 {
            let mut read = [0u8; TOUCHPOINT_ENTRY_LEN];
            self.read(i2c, GT911_TOUCHPOINT_1_REG, &mut read)?;
            let point = decode_point(&read);
            Some(point)
        } else {
            None
        };

        // clear status register
        self.write(i2c, GT911_TOUCHPOINT_STATUS_REG, 0)?;
        Ok(point)
    }

    /// Gets multiple stack allocated touch points (0-5 points)
    /// Returns points.len()==0 for release, points.len()>0 for press or move and Err(Error::NotReady) for no data
    pub fn get_multi_touch(
        &self,
        i2c: &mut I2C,
    ) -> Result<heapless::Vec<Point, MAX_NUM_TOUCHPOINTS>, Error<E>> {
        let num_touch_points = self.get_num_touch_points(i2c)?;

        let points = if num_touch_points > 0 {
            assert!(num_touch_points <= MAX_NUM_TOUCHPOINTS);
            let mut points = heapless::Vec::new();

            // read touch points
            let mut read = [0u8; TOUCHPOINT_ENTRY_LEN * MAX_NUM_TOUCHPOINTS];
            self.read(
                i2c,
                GT911_TOUCHPOINT_1_REG,
                &mut read[..TOUCHPOINT_ENTRY_LEN * num_touch_points],
            )?;

            for n in 0..num_touch_points {
                let start = n * TOUCHPOINT_ENTRY_LEN;
                let point = decode_point(&read[start..start + TOUCHPOINT_ENTRY_LEN]);
                points.push(point).ok();
            }

            points
        } else {
            heapless::Vec::new()
        };

        // clear status register
        self.write(i2c, GT911_TOUCHPOINT_STATUS_REG, 0)?;
        Ok(points)
    }

    fn get_num_touch_points(&self, i2c: &mut I2C) -> Result<usize, Error<E>> {
        // read coords
        let mut read = [0u8; 1];
        self.read(i2c, GT911_TOUCHPOINT_STATUS_REG, &mut read)?;

        let status = read[0];
        let ready = (status & 0x80) > 0;
        let num_touch_points = (status & 0x0F) as usize;

        if ready {
            Ok(num_touch_points)
        } else {
            Err(Error::NotReady)
        }
    }

    fn write(&self, i2c: &mut I2C, register: u16, value: u8) -> Result<(), Error<E>> {
        let register = register.to_be_bytes();
        let cmd = [register[0], register[1], value];
        i2c.write(self.i2c_addr, &cmd).map_err(Error::I2C)
    }

    fn read(&self, i2c: &mut I2C, register: u16, buf: &mut [u8]) -> Result<(), Error<E>> {
        i2c.write_read(self.i2c_addr, &register.to_be_bytes(), buf)
            .map_err(Error::I2C)
    }
}

/// Async Gt911
pub struct Gt911<I2C> {
    i2c_addr: u8, // e.g. 0x5D
    i2c: PhantomData<I2C>,
}

/// Use the default I2C address for communication
impl<I2C> Default for Gt911<I2C> {
    fn default() -> Self {
        Self {
            i2c_addr: GT911_I2C_ADDR_BA,
            i2c: PhantomData,
        }
    }
}

/// Async Gt911 implementation
impl<I2C, E> Gt911<I2C>
where
    I2C: embedded_hal_async::i2c::I2c<Error = E>,
{
    /// Creates a new instance with a user specified i2c address
    pub fn new(i2c_addr: u8) -> Self {
        Self {
            i2c_addr,
            i2c: PhantomData,
        }
    }

    /// Checks the ProductId for a "911\0" string response and resets the status register
    /// Only needs to be called once on startup
    pub async fn init(&self, i2c: &mut I2C) -> Result<(), Error<E>> {
        // switch to command mode
        self.write(i2c, GT911_COMMAND_REG, 0).await?;

        // read the product_id and confirm that it is expected
        let mut read = [0u8; 4];
        self.read(i2c, GT911_PRODUCT_ID_REG, &mut read).await?;
        match str::from_utf8(&read) {
            Ok(product_id) => {
                if product_id != "911\0" {
                    return Err(Error::UnexpectedProductId);
                }
            }
            Err(_) => {
                return Err(Error::UnexpectedProductId);
            }
        }

        // clear status register
        self.write(i2c, GT911_TOUCHPOINT_STATUS_REG, 0).await?;
        Ok(())
    }

    /// Gets a single touch point
    /// Returns Ok(None) for release, Some(point) for press or move and Err(Error::NotReady) for no data
    pub async fn get_touch(&self, i2c: &mut I2C) -> Result<Option<Point>, Error<E>> {
        let num_touch_points = self.get_num_touch_points(i2c).await?;

        let point = if num_touch_points > 0 {
            let mut read = [0u8; TOUCHPOINT_ENTRY_LEN];
            self.read(i2c, GT911_TOUCHPOINT_1_REG, &mut read).await?;
            let point = decode_point(&read);
            Some(point)
        } else {
            None
        };

        // clear status register
        self.write(i2c, GT911_TOUCHPOINT_STATUS_REG, 0).await?;
        Ok(point)
    }

    /// Gets multiple stack allocated touch points (0-5 points)
    /// Returns points.len()==0 for release, points.len()>0 for press or move and Err(Error::NotReady) for no data
    pub async fn get_multi_touch(
        &self,
        i2c: &mut I2C,
    ) -> Result<heapless::Vec<Point, MAX_NUM_TOUCHPOINTS>, Error<E>> {
        let num_touch_points = self.get_num_touch_points(i2c).await?;

        let points = if num_touch_points > 0 {
            assert!(num_touch_points <= MAX_NUM_TOUCHPOINTS);
            let mut points = heapless::Vec::new();

            // read touch points
            let mut read = [0u8; TOUCHPOINT_ENTRY_LEN * MAX_NUM_TOUCHPOINTS];
            self.read(
                i2c,
                GT911_TOUCHPOINT_1_REG,
                &mut read[..TOUCHPOINT_ENTRY_LEN * num_touch_points],
            )
            .await?;

            for n in 0..num_touch_points {
                let start = n * TOUCHPOINT_ENTRY_LEN;
                let point = decode_point(&read[start..start + TOUCHPOINT_ENTRY_LEN]);
                points.push(point).ok();
            }

            points
        } else {
            heapless::Vec::new()
        };

        // clear status register
        self.write(i2c, GT911_TOUCHPOINT_STATUS_REG, 0).await?;
        Ok(points)
    }

    async fn get_num_touch_points(&self, i2c: &mut I2C) -> Result<usize, Error<E>> {
        // read coords
        let mut read = [0u8; 1];
        self.read(i2c, GT911_TOUCHPOINT_STATUS_REG, &mut read)
            .await?;

        let status = read[0];
        let ready = (status & 0x80) > 0;
        let num_touch_points = (status & 0x0F) as usize;

        if ready {
            Ok(num_touch_points)
        } else {
            Err(Error::NotReady)
        }
    }

    async fn write(&self, i2c: &mut I2C, register: u16, value: u8) -> Result<(), Error<E>> {
        let register = register.to_be_bytes();
        let cmd = [register[0], register[1], value];
        i2c.write(self.i2c_addr, &cmd).await.map_err(Error::I2C)
    }

    async fn read(&self, i2c: &mut I2C, register: u16, buf: &mut [u8]) -> Result<(), Error<E>> {
        i2c.write_read(self.i2c_addr, &register.to_be_bytes(), buf)
            .await
            .map_err(Error::I2C)
    }
}

fn decode_point(buf: &[u8]) -> Point {
    assert!(buf.len() >= TOUCHPOINT_ENTRY_LEN);
    Point {
        track_id: buf[0],
        x: u16::from_le_bytes([buf[1], buf[2]]),
        y: u16::from_le_bytes([buf[3], buf[4]]),
        area: u16::from_le_bytes([buf[5], buf[6]]),
        // NOTE: the last byte is reserved
    }
}
