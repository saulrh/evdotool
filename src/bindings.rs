use crate::device;
use crate::time_util;
use anyhow::Context;
use evdev_rs::enums::EventCode;

pub const BINDINGS_NAME: &str = "bindings";
pub const IS_BOUND_KEY: &str = "DEVICE_IS_BOUND";

use crate::DeviceContext;

fn device_key(dev: &DeviceContext) -> rlua::Result<String> {
    Ok(String::from(dev.friendly_name()?))
}

pub fn set_up_bindings<'lua>(ctx: &rlua::Context<'lua>) -> rlua::Result<()> {
    let t = ctx.create_table()?;
    for dev in device::DeviceContext::list_all(time_util::CLOCK)?.iter() {
        let dev_name = device_key(dev)?;
        let dev_table = ctx.create_table()?;
        dev_table.set(IS_BOUND_KEY, false)?;
        t.set(dev_name, dev_table)?;
    }
    ctx.set_named_registry_value(BINDINGS_NAME, t)?;
    Ok(())
}

pub fn device_has_bindings(ctx: &rlua::Context, dev: &DeviceContext) -> rlua::Result<bool> {
    let t = ctx
        .named_registry_value::<str, rlua::Table>(BINDINGS_NAME)
        .with_context(|| "getting registry value")
        .map_err(rlua::Error::external)?;
    let dev_name = device_key(dev)?;
    let dev_table = t
        .get::<String, rlua::Table>(dev_name)
        .with_context(|| "getting device table")
        .map_err(rlua::Error::external)?;
    dev_table.get(IS_BOUND_KEY)
}

pub fn set_in_bindings_table<'lua, 'a>(
    ctx: &rlua::Context<'lua>,
    dev_ud: &rlua::AnyUserData<'lua>,
    event: &EventCode,
    callback: rlua::Function<'lua>,
) -> rlua::Result<()> {
    let s = event.to_string();
    let t = ctx.named_registry_value::<&str, rlua::Table>(&BINDINGS_NAME)?;
    let dev_name = device_key(&dev_ud.borrow::<DeviceContext>().unwrap())?;
    let dev_table = t.get::<String, rlua::Table>(dev_name.clone())?;
    dev_table.set(s, callback)?;
    dev_table.set(IS_BOUND_KEY, true)?;
    t.set(dev_name.clone(), dev_table)?;
    ctx.set_named_registry_value(BINDINGS_NAME, t)?;
    Ok(())
}

pub fn get_in_bindings_map<'a, 'lua>(
    ctx: &rlua::Context<'lua>,
    dev: &DeviceContext,
    event: &EventCode,
) -> rlua::Result<Option<rlua::Function<'lua>>> {
    let s = event.to_string();
    let t = ctx.named_registry_value::<&str, rlua::Table>(&BINDINGS_NAME)?;
    let dev_name = device_key(dev)?;
    let dev_table = t.get::<String, rlua::Table>(dev_name)?;
    match dev_table.get::<_, rlua::Value>(s)? {
        rlua::Value::Nil => Ok(None),
        rlua::Value::Function(f) => Ok(Some(f)),
        other => panic!("bindings table contained the wrong thing: `{:?}`", other),
    }
}
