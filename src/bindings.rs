use evdev_rs::enums::EventCode;
use std::collections::HashMap;

pub type BindingsMap<'lua> = HashMap<String, rlua::Function<'lua>>;

pub fn bindings_map_from<'lua>(t: rlua::Table<'lua>) -> rlua::Result<BindingsMap<'lua>> {
    Ok(t.pairs::<String, rlua::Function>()
        .collect::<rlua::Result<BindingsMap>>()?)
}

pub fn set_in_bindings_table<'lua>(
    table: &mut rlua::Table<'lua>,
    event: &EventCode,
    callback: rlua::Function<'lua>,
) -> rlua::Result<()> {
    let s = event.to_string();
    table.set(s, callback)?;
    Ok(())
}

pub fn get_in_bindings_map<'a, 'lua>(
    map: &'a BindingsMap<'lua>,
    event: &EventCode,
) -> Option<&'a rlua::Function<'lua>> {
    let s = event.to_string();
    map.get(&s)
}
