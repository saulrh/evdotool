use evdev_rs::enums::EventCode;
use evdev_rs::util::event_code_to_int;
use std::collections::HashMap;

pub type BindingsMap<'lua> = HashMap<u32, HashMap<u32, rlua::Function<'lua>>>;

pub fn bindings_map_from<'lua>(t: rlua::Table<'lua>) -> rlua::Result<BindingsMap<'lua>> {
    t.pairs::<u32, rlua::Table>()
        .collect::<rlua::Result<Vec<_>>>()?
        .into_iter()
        .map(|(k, v)| {
            Ok((
                k,
                v.pairs::<u32, rlua::Function>()
                    .collect::<rlua::Result<HashMap<_, _>>>()?,
            ))
        })
        .collect()
}

pub fn set_in_bindings_table<'lua>(
    ctx: &rlua::Context<'lua>,
    table: &mut rlua::Table<'lua>,
    event: &EventCode,
    callback: rlua::Function<'lua>,
) -> rlua::Result<()> {
    let (raw_event_type, raw_event_code) = event_code_to_int(event);
    let type_table = match table.get(raw_event_type)? {
        rlua::Value::Nil => ctx.create_table()?,
        rlua::Value::Table(t) => t,
        _ => panic!("bindings table had a non-table value at the top level"),
    };
    type_table.set(raw_event_code, callback)?;
    table.set(raw_event_type, type_table)?;
    Ok(())
}

pub fn get_in_bindings_map<'a, 'lua>(
    map: &'a BindingsMap<'lua>,
    event: &EventCode,
) -> Option<&'a rlua::Function<'lua>> {
    let (raw_event_type, raw_event_code) = event_code_to_int(event);
    map.get(&raw_event_type)
        .map(|codes| codes.get(&raw_event_code))
        .flatten()
}
