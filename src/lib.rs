use mlua::prelude::*;

mod fuzzy;

pub fn fuzzy<'a>(lua: &'a Lua, params: (String, Vec<String>)) -> LuaResult<LuaTable<'a>> {
    dbg!(&params);
    let result = fuzzy::fuzzy_match(&params.0, params.1, true);
    dbg!(&result);
    lua.create_table_from(result)
    // Ok(LuaMultiValue::from_vec(fuzzy_match(params.0, params.1, true)))
}


#[mlua::lua_module]
fn nucleo_nvim(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("fuzzy_match", lua.create_function(fuzzy)?)?;
    Ok(exports)
}
