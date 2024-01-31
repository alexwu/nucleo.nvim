use mlua::prelude::*;
use serde::Deserialize;

pub fn call_or_get<T>(lua: &Lua, val: LuaValue, field: &str) -> LuaResult<T>
where
    T: for<'a> IntoLua<'a> + for<'a> FromLua<'a> + for<'a> Deserialize<'a>,
{
    let table = LuaTable::from_lua(val, lua)?;
    match table.get(field)? {
        LuaValue::Function(func) => {
            log::debug!("in the function section");
            func.call::<_, T>(())
        }
        val => {
            log::debug!("val: {:?}", &val);
            lua.from_value(val)
        }
    }
}
