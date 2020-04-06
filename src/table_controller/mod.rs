mod signal_decoder;
use signal_decoder::SignalDecoder;
use sysfs_gpio::{Edge, Error, Pin, PinPoller};

pub struct TableController {
  is_auto_mode: bool,
  direction: Direction,
  target_height: Option<i32>,
  signal_decoder: SignalDecoder,
  signal_motor: Pin,
  signal_controller: Pin,
  up_motor: Pin,
  up_controller: Pin,
  down_motor: Pin,
  down_controller: Pin,
}

pub struct ControlPins {
  pub up_motor_pin: u64,
  pub up_controller_pin: u64,
  pub down_motor_pin: u64,
  pub down_controller_pin: u64,
  pub signal_motor_pin: u64,
  pub signal_controller_pin: u64,
}

impl TableController {
  pub fn new(control_pins: ControlPins) -> TableController {
    match TableController::initialize(control_pins) {
      Ok((
        signal_motor,
        signal_controller,
        up_motor,
        up_controller,
        down_motor,
        down_controller,
      )) => TableController {
        is_auto_mode: true,
        direction: Direction::None,
        target_height: Option::None,
        signal_decoder: SignalDecoder::new(),
        signal_motor,
        signal_controller,
        up_motor,
        up_controller,
        down_motor,
        down_controller,
      },
      Err(_error) => panic!("Failed to initialize table"),
    }
  }

  pub fn initialize(pins: ControlPins) -> Result<(Pin, Pin, Pin, Pin, Pin, Pin), Error> {
    let signal_motor = Pin::new(pins.signal_motor_pin);
    let signal_controller = Pin::new(pins.signal_controller_pin);
    let up_motor = Pin::new(pins.up_motor_pin);
    let up_controller = Pin::new(pins.up_controller_pin);
    let down_motor = Pin::new(pins.down_motor_pin);
    let down_controller = Pin::new(pins.down_controller_pin);
    signal_motor.export()?;
    signal_controller.export()?;
    up_motor.export()?;
    up_controller.export()?;
    down_motor.export()?;
    down_controller.export()?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    signal_motor.set_direction(sysfs_gpio::Direction::In)?;
    signal_motor.set_edge(Edge::BothEdges)?; // we receive interrupts here

    signal_controller.set_direction(sysfs_gpio::Direction::Out)?;

    up_motor.set_direction(sysfs_gpio::Direction::Out)?;
    up_controller.set_direction(sysfs_gpio::Direction::In)?;
    down_motor.set_direction(sysfs_gpio::Direction::Out)?;
    down_controller.set_direction(sysfs_gpio::Direction::In)?;

    Ok((
      signal_motor,
      signal_controller,
      up_motor,
      up_controller,
      down_motor,
      down_controller,
    ))
  }

  pub fn tick(self: &mut Self) -> Result<(), Error> {
    self.handle_current_signal_bit()?;
    self.handle_switch_inputs()?;
    self.control_table_movement()?;
    Ok(())
  }

  fn handle_current_signal_bit(self: &mut Self) -> Result<(), Error> {
    let current_bit = self.signal_motor.get_value()?;
    self.signal_controller.set_value(current_bit)?;
    self.signal_decoder.process_bit(current_bit);
    Ok(())
  }

  fn handle_switch_inputs(self: &mut Self) -> Result<(), Error> {
    let up_switch_pressed = self.up_controller.get_value()?;
    let down_switch_pressed = self.down_controller.get_value()?;

    if up_switch_pressed > 0 || down_switch_pressed > 0 {
      self.is_auto_mode = false;
      self.up_motor.set_value(up_switch_pressed)?;
      self.down_motor.set_value(down_switch_pressed)?;
    } else if !self.is_auto_mode {
      self.up_motor.set_value(up_switch_pressed)?;
      self.down_motor.set_value(down_switch_pressed)?;
    }
    Ok(())
  }

  fn control_table_movement(self: &mut Self) -> Result<(), Error> {
    if !self.is_auto_mode {
      return Ok(())
    }

    if self.should_move_up() {
      return self.move_table_up();
    } else if self.shloud_move_down() {
      return self.move_table_down();
    } else {
      return self.stop_table();
    }
  }

  fn should_move_up(&self) -> bool {
    return self.direction == Direction::Up
        && self.signal_decoder.current_height.is_some()
        && self.target_height.is_some()
        && self.signal_decoder.current_height.unwrap() < self.target_height.unwrap();
  }

  fn shloud_move_down(&self) -> bool {
    return self.direction == Direction::Down
        && self.signal_decoder.current_height.is_some()
        && self.target_height.is_some()
        && self.signal_decoder.current_height.unwrap() > self.target_height.unwrap();
  }

  fn move_table_down(&self) -> Result<(), Error> {
    self.down_motor.set_value(1)?;
    self.up_motor.set_value(0)?;
    Ok(())
  }

  fn move_table_up(&self) -> Result<(), Error> {
    self.down_motor.set_value(0)?;
    self.up_motor.set_value(1)?;
    Ok(())
  }

  fn stop_table(&self) -> Result<(), Error> {
    self.down_motor.set_value(0)?;
    self.up_motor.set_value(0)?;
    Ok(())
  }

  pub fn wait_for_interrupt(&self) -> Result<PinPoller, Error> {
    self.signal_motor.get_poller()
  }
}

#[derive(Eq, PartialEq)]
enum Direction {
  Up,
  Down,
  None,
}
