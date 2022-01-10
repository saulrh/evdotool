use crate::device::DeviceContext;
use crate::evdev_util;
use rlua::prelude::{LuaContext, LuaResult};

use crate::bindings::set_in_bindings_table;
use crate::time_util;

pub fn make_sleep(ctx: &LuaContext) -> LuaResult<()> {
    ctx.globals().set(
        "sleep",
        ctx.create_function(|_, secs: f64| {
            std::thread::sleep(std::time::Duration::from_secs_f64(secs));
            Ok(())
        })?,
    )
}

pub fn make_bind<'a>(ctx: &LuaContext) -> LuaResult<()> {
    ctx.globals().set(
        "bind",
        ctx.create_function(
            move |ctx, (dev_handle, event, callback): (rlua::AnyUserData, String, rlua::Function)| {
                let event_code =
                    evdev_util::event_code_from_str(event).map_err(rlua::Error::external)?;
                set_in_bindings_table(&ctx, &dev_handle, &event_code, callback)?;
                Ok(())
            },
        )?,
    )
}

pub fn make_all_event_codes(ctx: &LuaContext) -> LuaResult<()> {
    ctx.globals().set(
        "CODES",
        ctx.create_sequence_from(evdev_util::all_event_codes().map(|ec| ec.to_string()))?,
    )
}

pub fn make_device_userdatas(ctx: &LuaContext) -> LuaResult<()> {
    ctx.globals().set(
        "DEVICES",
        DeviceContext::list_all_as_userdata(time_util::CLOCK, ctx)?,
    )
}
