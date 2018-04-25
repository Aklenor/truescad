use hlua;
use hlua::{Lua, LuaError};
use lobject::LObject;
use lobject_vector::LObjectVector;
use printbuffer;
use sandbox;
use truescad_types::Float;

pub const USER_FUNCTION_NAME: &'static str = "__luscad_user_function__";
pub const SANDBOX_ENV_NAME: &'static str = "__luascad_sandbox_env__";

pub fn eval(
    script: &str,
) -> Result<(String, Option<Box<::truescad_primitive::Object<Float>>>), LuaError> {
    let mut result = None;
    let print_output;
    {
        let mut lua = Lua::new();
        lua.openlibs();
        sandbox::set_sandbox_env(&mut lua, SANDBOX_ENV_NAME);
        let printbuffer =
            printbuffer::PrintBuffer::new_and_expose_to_lua(&mut lua, SANDBOX_ENV_NAME);
        {
            let mut sandbox_env = lua.get::<hlua::LuaTable<_>, _>(SANDBOX_ENV_NAME).unwrap();
            LObject::export_factories(&mut sandbox_env, printbuffer.get_tx());
            sandbox_env.set(
                "build",
                hlua::function1(|o: &LObject| result = o.into_object()),
            );
        }
        // LObjectVector needs access to full lua object and the SANDBOX_ENV_NAME.
        LObjectVector::export_factories(&mut lua, SANDBOX_ENV_NAME);

        // Store the script in the Lua var USER_FUNCTION_NAME.
        try!(lua.checked_set(USER_FUNCTION_NAME, hlua::LuaCode(script)));
        // Use this script wrapper to execute USER_FUNCTION_NAME with sandbox env.
        try!(lua.execute::<()>(&format!(
            "debug.setupvalue({}, 1, {}); return {}();",
            USER_FUNCTION_NAME,
            SANDBOX_ENV_NAME,
            USER_FUNCTION_NAME
        )));
        print_output = printbuffer.get_buffer();
    }
    return Ok((print_output, result));
}
