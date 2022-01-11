use crate::evdev_util;
use crate::friendly_name::friendly_name;
use evdev_rs::enums::{EventCode, EV_ABS};
use evdev_rs::{Device, DeviceWrapper, InputEvent};
use rlua::{ToLua, UserData, UserDataMethods};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::result::Result;
use std::str::FromStr;

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

use crate::time_util::Clock;

#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("`{0}` is not a valid evdev event code")]
    InvalidEventCode(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    FromLua(#[from] rlua::Error),
}

impl From<DeviceError> for rlua::Error {
    fn from(se: DeviceError) -> rlua::Error {
        rlua::Error::external(se)
    }
}

pub type DeviceResult<T> = Result<T, DeviceError>;

#[derive(Debug)]
pub struct DeviceContext {
    dev: Device,
}

impl Hash for DeviceContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dev.uniq().hash(state);
        self.dev.name().hash(state);
        self.dev.product_id().hash(state);
        self.dev.vendor_id().hash(state);
    }
}

impl DeviceContext {
    pub fn new(dev: Device) -> Self {
        Self { dev }
    }

    pub fn friendly_name(&self) -> DeviceResult<&str> {
        Ok(friendly_name(self))
    }

    pub fn get_capabilities(&self) -> DeviceResult<HashSet<EventCode>> {
        Ok(evdev_util::all_event_codes()
            .filter(|ec| self.dev.has(*ec))
            .collect())
    }

    pub fn list_all(clock: Clock) -> DeviceResult<Vec<Self>> {
        let mut js: Vec<Device> = std::fs::read_dir("/dev/input")?
            .filter_map(Result::ok)
            .map(|de| de.path())
            .map(std::fs::File::open)
            .filter_map(Result::ok)
            .map(Device::new_from_file)
            .filter_map(Result::ok)
            .collect();
        for j in js.iter_mut() {
            j.set_clock_id(clock.raw_id())?;
        }
        Ok(js.into_iter().map(Self::new).collect())
    }

    pub fn list_all_as_userdata<'a>(
        clock: Clock,
        ctx: &rlua::Context<'a>,
    ) -> rlua::Result<Vec<rlua::AnyUserData<'a>>> {
        let uds = Self::list_all(clock)?
            .into_iter()
            .map(|d| ctx.create_userdata(d))
            .collect::<Result<Vec<rlua::AnyUserData>, rlua::Error>>()?;
        for ud in uds.iter() {
            ud.set_user_value(ctx.create_table()?)?;
        }
        Ok(uds)
    }

    pub fn next_event(&self) -> DeviceResult<InputEvent> {
        let (_, input_event) = self
            .dev
            .next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING)?;
        Ok(input_event)
    }

    pub fn raw_fd(&self) -> RawFd {
        self.dev.file().as_raw_fd()
    }
}

impl UserData for DeviceContext {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("friendly_name", |ctx, this, _: ()| {
            this.friendly_name().map(|s| s.to_lua(ctx))?
        });

        methods.add_method("name", |_, this, _: ()| {
            Ok(this.dev.name().map(String::from))
        });

        methods.add_method("uniq", |_, this, _: ()| {
            Ok(this.dev.uniq().map(String::from))
        });

        methods.add_method("product_id", |_, this, _: ()| Ok(this.dev.product_id()));

        methods.add_method("vendor_id", |_, this, _: ()| Ok(this.dev.vendor_id()));

        methods.add_method("axis_info", |ctx, this, axis: String| {
            let code = &EventCode::EV_ABS(
                // <EV_ABS as FromStr>::Err is just (), so not only do
                // we not need to use or hold on to the error for more
                // info, we *can't* store it
                EV_ABS::from_str(&axis).map_err(|_| DeviceError::InvalidEventCode(axis))?,
            );
            match this.dev.abs_info(code) {
                Some(info) => {
                    let t = ctx.create_table()?;
                    t.set("value", info.value)?;
                    t.set("minimum", info.minimum)?;
                    t.set("maximum", info.maximum)?;
                    t.set("fuzz", info.fuzz)?;
                    t.set("flat", info.flat)?;
                    t.set("resolution", info.resolution)?;
                    Ok(rlua::Value::Table(t))
                }
                None => Ok(rlua::Value::Nil),
            }
        });

        methods.add_method("get_caps", |ctx, this, _: ()| {
            ctx.create_sequence_from(
                this.get_capabilities()?
                    .into_iter()
                    .map(|ec| ec.to_string()),
            )
        });
    }
}
