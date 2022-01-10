use crate::interpolator::Interpolator;
use crate::time_util::{Clock, Time};
use evdev_rs::enums::{BusType, EventCode, EventType, EV_KEY, EV_REL, EV_SYN};
use evdev_rs::{DeviceWrapper, InputEvent, UInputDevice, UninitDevice};
use rlua::prelude::LuaError;
use rlua::{UserData, UserDataMethods};
use std::sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::evdev_util;

#[derive(thiserror::Error, Debug)]
pub enum InputError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ClockError(#[from] nix::errno::Errno),
    #[error(transparent)]
    WorkerDisconnected(#[from] SendError<WorkerMessage>),
    #[error("`{0}` is not a valid evdev key code")]
    InvalidKeyCode(String),
}

impl From<InputError> for LuaError {
    fn from(e: InputError) -> LuaError {
        LuaError::external(e)
    }
}

pub type InputResult<T> = Result<T, InputError>;

// max time between updates
const MOUSE_PERIOD: Duration = Duration::from_millis(1000 / 120);

#[derive(Copy, Clone, Debug)]
pub enum InputOp {
    Button { key: EV_KEY, value: i32 },
    XAbs { x: f64 },
    YAbs { y: f64 },
    XRel { dx: f64 },
    YRel { dy: f64 },
    XVel { dxdt: f64 },
    YVel { dydt: f64 },
}

#[derive(Copy, Clone, Debug)]
pub struct WorkerMessage(Time, InputOp);

#[derive(Debug)]
pub struct VirtualInput {
    sender: Sender<WorkerMessage>,
    clock: Clock,
}

impl VirtualInput {
    pub fn new(clock: Clock) -> InputResult<Self> {
        let dev = UninitDevice::new().unwrap();
        dev.set_name("evdotool virtual input");
        dev.set_bustype(BusType::BUS_USB as u16);
        dev.set_vendor_id(0xabcd);
        dev.set_product_id(0xefef);

        // Enable keyboard keys and mouse buttons

        // For some reason wlroots refuses to recognize our device if
        // we have too many events enabled, so we have to be
        // selective. >_<

        // We have to be sure we have these enabled, otherwise wlroots
        // or sway won't think we're a mouse and will ignore our axis
        // movements.
        dev.enable_event_type(&EventType::EV_KEY)?;
        dev.enable_event_code(&EventCode::EV_KEY(EV_KEY::BTN_LEFT), None)?;
        dev.enable_event_code(&EventCode::EV_KEY(EV_KEY::BTN_RIGHT), None)?;

        // Enable as much of the keyboard stuff as we can.
        evdev_util::codes_for(EventType::EV_KEY)
            .unwrap()
            .take(200)
            .map(|code| dev.enable_event_code(&code, None))
            .collect::<std::io::Result<()>>()?;

        // Enable relative mouse movements
        dev.enable_event_type(&EventType::EV_REL)?;
        dev.enable_event_code(&EventCode::EV_REL(EV_REL::REL_X), None)?;
        dev.enable_event_code(&EventCode::EV_REL(EV_REL::REL_Y), None)?;

        // Enable syn reports
        dev.enable_event_code(&EventCode::EV_SYN(EV_SYN::SYN_REPORT), None)?;

        // Create the UInputDevice
        let device = UInputDevice::create_from_device(&dev)?;

        // Create channels
        let (sender, receiver) = channel();

        // Create and start the worker
        let mut worker = VirtualInputWorker {
            device: device,
            period: MOUSE_PERIOD,
            receiver: receiver,
            x_interp: None,
            y_interp: None,
            clock: clock,
        };
        thread::spawn(move || {
            worker.run();
        });

        // Return ourself
        Ok(VirtualInput { sender, clock })
    }

    fn time_or_now(&self, time: Option<Time>) -> InputResult<Time> {
        Ok(match time {
            Some(t) => t,
            None => self.clock.now()?,
        })
    }

    fn send(&self, time: Option<Time>, op: InputOp) -> InputResult<()> {
        self.sender
            .send(WorkerMessage(self.time_or_now(time)?, op))?;
        Ok(())
    }

    pub fn button(&self, time: Option<Time>, key: EV_KEY, value: i32) -> InputResult<()> {
        self.send(
            time,
            InputOp::Button {
                key: key,
                value: value,
            },
        )
    }

    pub fn set_x_vel(&self, time: Option<Time>, dxdt: f64) -> InputResult<()> {
        self.send(time, InputOp::XVel { dxdt })
    }

    pub fn set_y_vel(&self, time: Option<Time>, dydt: f64) -> InputResult<()> {
        self.send(time, InputOp::YVel { dydt })
    }

    pub fn move_x(&self, time: Option<Time>, dx: f64) -> InputResult<()> {
        self.send(time, InputOp::XRel { dx })
    }

    pub fn move_y(&self, time: Option<Time>, dy: f64) -> InputResult<()> {
        self.send(time, InputOp::YRel { dy })
    }

    pub fn set_x(&self, time: Option<Time>, x: f64) -> InputResult<()> {
        self.send(time, InputOp::XAbs { x })
    }

    pub fn set_y(&self, time: Option<Time>, y: f64) -> InputResult<()> {
        self.send(time, InputOp::YAbs { y })
    }
}

impl UserData for VirtualInput {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("set_x_vel", |_, this, (dxdt, t): (f64, Option<Time>)| {
            this.set_x_vel(t, dxdt).map_err(|e| LuaError::external(e))
        });
        methods.add_method("set_y_vel", |_, this, (dydt, t): (f64, Option<Time>)| {
            this.set_y_vel(t, dydt).map_err(|e| LuaError::external(e))
        });
        methods.add_method("move_x", |_, this, (dx, t): (f64, Option<Time>)| {
            this.move_x(t, dx).map_err(|e| LuaError::external(e))
        });
        methods.add_method("move_y", |_, this, (dy, t): (f64, Option<Time>)| {
            this.move_y(t, dy).map_err(|e| LuaError::external(e))
        });
        methods.add_method("set_x", |_, this, (x, t): (f64, Option<Time>)| {
            this.set_x(t, x).map_err(|e| LuaError::external(e))
        });
        methods.add_method("set_y", |_, this, (y, t): (f64, Option<Time>)| {
            this.set_y(t, y).map_err(|e| LuaError::external(e))
        });
        methods.add_method(
            "button",
            |_, this, (key_string, value, t): (String, i32, Option<Time>)| {
                // <EV_ABS as FromStr>::Err is just (), so not only do
                // we not need to use or hold on to the error for more
                // info, we *can't* store it
                let key = key_string
                    .parse()
                    .map_err(|_| InputError::InvalidKeyCode(key_string))?;

                this.button(t, key, value)
                    .map_err(|e| LuaError::external(e))
            },
        );
    }
}

#[derive(Debug)]
struct VirtualInputWorker {
    device: UInputDevice,
    period: Duration,
    receiver: Receiver<WorkerMessage>,
    x_interp: Option<Interpolator>,
    y_interp: Option<Interpolator>,
    clock: Clock,
}

impl VirtualInputWorker {
    fn run(&mut self) {
        loop {
            std::thread::sleep(self.period);
            loop {
                match self.receiver.try_recv() {
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                    Ok(msg) => self.process(msg).unwrap(),
                };
            }
            let t = self.clock.now().expect("Failed to get the time");
            self.send_velocity(t).unwrap();
        }
    }

    fn process(&mut self, WorkerMessage(time, op): WorkerMessage) -> InputResult<()> {
        match op {
            InputOp::Button { key, value } => self.button(time, key, value)?,
            InputOp::XAbs { x } => {
                self.write_x_move(time - Time::from(MOUSE_PERIOD / 2), f64::MIN)?;
                self.write_x_move(time, x)?;
                self.syn(time)?;
            }
            InputOp::YAbs { y } => {
                self.write_x_move(time - Time::from(MOUSE_PERIOD / 2), f64::MIN)?;
                self.write_x_move(time, y)?;
                self.syn(time)?;
            }
            InputOp::XRel { dx } => {
                self.write_x_move(time, dx)?;
                self.syn(time)?;
            }
            InputOp::YRel { dy } => {
                self.write_y_move(time, dy)?;
                self.syn(time)?;
            }
            InputOp::XVel { dxdt } => self.set_x_vel(time, dxdt)?,
            InputOp::YVel { dydt } => self.set_y_vel(time, dydt)?,
        };
        Ok(())
    }

    fn syn(&self, time: Time) -> InputResult<()> {
        self.device.write_event(&InputEvent {
            time: time.into(),
            event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
            value: 0,
        })?;
        Ok(())
    }

    fn button(&self, time: Time, key: EV_KEY, value: i32) -> InputResult<()> {
        self.device.write_event(&InputEvent {
            time: time.into(),
            event_code: EventCode::EV_KEY(key),
            value: value,
        })?;
        self.syn(time)?;
        Ok(())
    }

    fn set_x_vel(&mut self, time: Time, x: f64) -> InputResult<()> {
        if x == 0.0 {
            self.x_interp = None;
        } else {
            if self.x_interp.is_none() {
                self.x_interp = Some(Interpolator::new(time));
            }
            (&mut self.x_interp).as_mut().unwrap().update(time, x);
        }
        Ok(())
    }

    fn set_y_vel(&mut self, time: Time, y: f64) -> InputResult<()> {
        if y == 0.0 {
            self.y_interp = None;
        } else {
            if self.y_interp.is_none() {
                self.y_interp = Some(Interpolator::new(self.clock.now()?));
            }
            (&mut self.y_interp).as_mut().unwrap().update(time, y);
        }
        Ok(())
    }

    fn write_x_move(&mut self, time: Time, x: f64) -> InputResult<()> {
        self.device.write_event(&InputEvent {
            time: time.into(),
            event_code: EventCode::EV_REL(EV_REL::REL_X),
            value: x as i32,
        })?;
        Ok(())
    }

    fn write_y_move(&mut self, time: Time, y: f64) -> InputResult<()> {
        self.device.write_event(&InputEvent {
            time: time.into(),
            event_code: EventCode::EV_REL(EV_REL::REL_Y),
            value: y as i32,
        })?;
        Ok(())
    }

    fn send_velocity(&mut self, t: Time) -> InputResult<()> {
        if let Some(interp) = &mut self.x_interp {
            let dx = interp.interpolate(t);
            self.write_x_move(t, dx as f64)?;
        }
        if let Some(interp) = &mut self.y_interp {
            let dy = interp.interpolate(t);
            self.write_y_move(t, dy as f64)?;
        }
        self.syn(t)?;
        Ok(())
    }
}
