use core::time::Duration;
use derive_more::{Add, Constructor, From, Mul, Sub};
use evdev_rs::TimeVal;
use nix::sys::time::TimeSpec;
use nix::time::ClockId;
use rlua::prelude::{LuaContext, LuaError, LuaResult, LuaValue};
use rlua::{FromLua, ToLua};

// This is the clock that everything in the program will use.
pub const CLOCK: Clock = Clock(nix::time::ClockId::CLOCK_MONOTONIC);

#[derive(Copy, Clone, Debug, From, Constructor)]
pub struct Clock(ClockId);

impl Clock {
    pub fn now(&self) -> nix::Result<Time> {
        Ok(self.0.now()?.into())
    }

    pub fn raw_id(&self) -> i32 {
        self.0.as_raw()
    }
}

#[derive(Copy, Clone, Add, Sub, Mul)]
/// Opaque duration struct. Usually used to represent instants by
/// holding seconds-since-epoch on CLOCK_MONOTONIC, but since we're
/// operating with two different c-ish time structs (evdev timevals
/// and linux timespecs) we often have to follow them and use this to
/// represent durations as well.
pub struct Time(
    /// Seconds
    f64,
);

impl From<f64> for Time {
    fn from(seconds: f64) -> Self {
        Self(seconds)
    }
}

impl From<Time> for f64 {
    fn from(time: Time) -> Self {
        time.0
    }
}

impl std::fmt::Debug for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        let local_now: chrono::DateTime<chrono::Local> = chrono::Local::now();
        let monotonic_now = Clock::new(nix::time::ClockId::CLOCK_MONOTONIC)
            .now()
            .map_err(|_| std::fmt::Error)?;
        let monotonic_diff = monotonic_now - *self;
        let chrono_diff = chrono::Duration::from_std(Duration::from(monotonic_diff))
            .map_err(|_| std::fmt::Error)?;
        f.write_fmt(format_args!(
            "Time {{ {} ({}) }}",
            self.0,
            local_now - chrono_diff
        ))
    }
}

impl<'lua> FromLua<'lua> for Time {
    fn from_lua(lua_value: LuaValue<'lua>, _: LuaContext<'lua>) -> LuaResult<Self> {
        match lua_value {
            LuaValue::Number(n) => Ok(Time(n)),
            _ => Err(LuaError::external("wrong type")),
        }
    }
}

impl<'lua> ToLua<'lua> for Time {
    fn to_lua(self, _: LuaContext<'lua>) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::Number(self.0))
    }
}

impl From<TimeSpec> for Time {
    fn from(spec: TimeSpec) -> Self {
        Time::from(Duration::from(spec))
    }
}

impl From<TimeVal> for Time {
    fn from(tv: TimeVal) -> Self {
        Time(tv.tv_sec as f64 + ((tv.tv_usec / 10_000) as f64))
    }
}

impl From<Duration> for Time {
    fn from(duration: Duration) -> Self {
        Time(duration.as_secs_f64())
    }
}

impl From<Time> for TimeVal {
    fn from(time: Time) -> Self {
        TimeVal {
            tv_sec: time.0.floor() as i64,
            tv_usec: (time.0.fract() * 10_000f64) as i64,
        }
    }
}

impl From<Time> for TimeSpec {
    fn from(time: Time) -> Self {
        TimeSpec::from_duration(Duration::from(time))
    }
}

impl From<Time> for Duration {
    fn from(time: Time) -> Self {
        Duration::from_secs_f64(time.0)
    }
}
