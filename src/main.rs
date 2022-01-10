use anyhow::{Context, Result};
use nix::sys::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};
use rlua::prelude::LuaError;
use rlua::Lua;
use std::cell::Ref;
use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;

mod friendly_name;

mod lua_util;

mod evdev_util;

mod global_bindings;
use global_bindings::*;

mod device;
use device::DeviceContext;

mod interpolator;

mod time_util;

mod virtual_input;

#[derive(Debug, StructOpt)]
#[structopt(author, setting(AppSettings::TrailingVarArg))]
struct EvdotoolOpt {
    /// Run the given script
    #[structopt()]
    script: PathBuf,
    /// Further args for the script
    #[structopt()]
    script_args: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
enum EvdotoolError {
    #[error("no bindings were created")]
    NoBindings,
}

fn main() -> Result<()> {
    let lua = Lua::new();

    let opt: EvdotoolOpt = EvdotoolOpt::from_args();

    let script = std::fs::read_to_string(&opt.script)
        .with_context(|| format!("while reading script {}", opt.script.to_string_lossy()))?;

    let input = virtual_input::VirtualInput::new(time_util::CLOCK)?;

    lua.context(|lua_ctx| {
        make_sleep(&lua_ctx)?;
        make_bind(&lua_ctx)?;
        make_all_event_codes(&lua_ctx)?;

        let globals = lua_ctx.globals();

        globals.set(
            "DEVICES",
            DeviceContext::list_all_as_userdata(time_util::CLOCK, &lua_ctx)?,
        )?;
        globals.set("INPUT", input)?;

        Ok::<(), LuaError>(())
    })
    .with_context(|| "while building lua runtime environment")?;

    lua.context(|lua_ctx| {
        lua_ctx.globals().set("arg", opt.script_args)?;
        lua_ctx.load(&script).eval()?;
        Ok::<(), LuaError>(())
    })
    .with_context(|| "while running script")?;

    lua.context(|lua_ctx| -> rlua::Result<()> {
        #[derive(Debug)]
        struct BoundDevice<'a, 'b> {
            dev: Ref<'b, DeviceContext>,
            bindings: lua_util::BindingsMap<'a>,
        }
        let pollfd = epoll_create1(EpollCreateFlags::empty())
            .with_context(|| "in epoll_create1")
            .map_err(rlua::Error::external)?;
        let mut events = Vec::new();
        let device_userdatas = lua_ctx
            .globals()
            .get::<_, rlua::Table>("DEVICES")?
            .sequence_values::<rlua::AnyUserData>()
            .collect::<rlua::Result<Vec<rlua::AnyUserData>>>()?;
        // this has to be defined after device_userdatas because it
        // borrows an rlua::Function from it and drops happen in
        // reverse order
        let mut bound_devices: Vec<BoundDevice> = Vec::new();
        for device_user_data in device_userdatas.iter() {
            let bindings_table = device_user_data.get_user_value::<rlua::Table>()?;
            let bindings = lua_util::bindings_map_from(bindings_table)?;

            if bindings.len() == 0 {
                continue;
            }

            let bound_device = BoundDevice {
                dev: device_user_data.borrow::<DeviceContext>().unwrap(),
                bindings: bindings,
            };
            let mut ev = EpollEvent::new(EpollFlags::EPOLLIN, bound_devices.len() as u64);

            epoll_ctl(
                pollfd,
                EpollOp::EpollCtlAdd,
                bound_device.dev.raw_fd(),
                &mut ev,
            )
            .with_context(|| "in epoll_ctl")
            .map_err(rlua::Error::external)?;
            bound_devices.push(bound_device);
            events.push(ev);
        }

        if bound_devices.len() == 0 {
            return Err(rlua::Error::external(EvdotoolError::NoBindings));
        }

        loop {
            let _ = epoll_wait(pollfd, &mut events, 1000)
                .with_context(|| "in epoll_wait")
                .map_err(rlua::Error::external)?;
            for event in events.iter() {
                if event.events().contains(EpollFlags::EPOLLIN) {
                    let bound_device = &bound_devices[event.data() as usize];
                    let input = bound_device.dev.next_event()?;
                    if let Some(callback) =
                        lua_util::get_in_bindings_map(&bound_device.bindings, &input.event_code)
                    {
                        callback.call::<_, ()>(input.value)?;
                    }
                }
            }
        }
    })
    .with_context(|| "while running bindings")?;

    println!("Done!");

    Ok(())
}
