# Examples for GT911 touchscreen

The following examples target the `stm32u5g9j-dk2` development kit.

## Setup

Install cross compilation target
```
rustup target add thumbv8m.main-none-eabihf
```

Install `probe-rs` tool to program the device (and print logs to the console). The `force` just reinstalls it in case you have an old version. 
```
cargo install probe-rs-tools --force
```
