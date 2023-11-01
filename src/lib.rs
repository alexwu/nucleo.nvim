use mlua::prelude::*;

mod fuzzy;

pub fn files(lua: &Lua, params: String) -> LuaResult<LuaValue> {
    fuzzy::files::<String>(&params, true).into_lua(lua)
}

pub fn fuzzy(lua: &Lua, params: (String, Vec<String>)) -> LuaResult<LuaValue> {
    let mut result = fuzzy::fuzzy_match(&params.0, params.1, true);
    // result.sort_by(|a, b| a.1.cmp(&b.1));
    result.sort_by(|a, b| b.1.cmp(&a.1));

    let table = Vec::from_iter(result.into_iter()
            .map(|(item, _)| item));

    table.into_lua(lua)
}

#[mlua::lua_module]
fn nucleo_nvim(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("fuzzy_match", lua.create_function(fuzzy)?)?;
    exports.set("files", lua.create_function(files)?)?;
    // dbg!("here");
    Ok(exports)
}
