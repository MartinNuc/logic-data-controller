# logic-data-controller

Driver for controlling height adjustable table desk

## Architecture

### Table controller

- reads table height from signal cable from the motor and resends it to the handle switch
- stores target height
- when target height is set it turns on automatic mode and moves table to given height

### Signal decoder

- reads 32 bits chunks of signal
- decodes table height

### Web server

- REST API to control driver
- GET `/table` gives current height and target height
- PATCH `/table` - partial update for `target_height` - sets new target height

## Run

`cargo run`

`cargo build`
