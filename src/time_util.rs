use core::ops::{Add, Mul, Sub};
use core::time::Duration;
use evdev_rs::TimeVal;
use nix::sys::time::TimeSpec;
use nix::time::ClockId;
use paste::paste;
use rlua::prelude::{LuaContext, LuaError, LuaResult, LuaValue};
use rlua::{FromLua, ToLua};

pub const CLOCK: Clock = Clock {
    clock: nix::time::ClockId::CLOCK_MONOTONIC,
};

#[derive(Copy, Clone, Debug)]
pub struct Clock {
    pub clock: ClockId,
}

impl Clock {
    pub fn new(clock_id: ClockId) -> Self {
        Clock { clock: clock_id }
    }

    pub fn now(&self) -> nix::Result<Time> {
        Ok(self.clock.now()?.into())
    }

    pub fn raw_id(&self) -> i32 {
        self.clock.as_raw()
    }
}

#[derive(Copy, Clone)]
/// Opaque duration struct. Usually used to represent instants by
/// holding seconds-since-epoch on CLOCK_MONOTONIC, but since we're
/// operating with two *different* c-ish time structs (evdev timevals
/// and linux timespecs) we often have to use this to represent
/// durations as well.
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
    /// Friendly display for humans that *assumes* that this
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

macro_rules! impl_binary_op {
    ($trait:ident) => {
        impl $trait<Time> for Time {
            type Output = Time;
            paste! {
                fn [< $trait:lower >](self, rhs: Time) -> Self::Output {
                    Time(self.0. [< $trait:lower >] (rhs.0))
                }
            }
        }
    };
}

impl_binary_op!(Add);
impl_binary_op!(Sub);
impl_binary_op!(Mul);

macro_rules! impl_time_op {
    ($trait:ident<$lhs:ty, $rhs:ty>) => {
        impl $trait<$rhs> for $lhs {
            type Output = Time;
            paste! {
                fn [< $trait:lower >](self, rhs: $rhs) -> Self::Output {
                    Time::from(self). [< $trait:lower >] (Time::from(rhs))
                }
            }
        }
    };
}

macro_rules! impl_time_ops {
    ($other:ty) => {
        impl_time_op!(Add<$other, Time>);
        impl_time_op!(Add<Time, $other>);
        impl_time_op!(Sub<$other, Time>);
        impl_time_op!(Sub<Time, $other>);
        impl_time_op!(Mul<$other, Time>);
        impl_time_op!(Mul<Time, $other>);
    }
}

impl_time_ops!(f64);
impl_time_ops!(Duration);
