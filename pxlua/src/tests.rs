
use mlua::Lua;
use super::error::Error;
use super::std::net::LuaTcpStream;
use tokio::net::TcpStream;
use tokio::task;

#[test]
/// Tests if tcp is found in lua, and if the function shim can be ran in rust
fn tcp_shim_access_thread() -> Result<(), Error> {
    let lua = Lua::new();

    lua.globals().set("", "");

    task::spawn_local(async move {    
        let func = lua.load(
            r#"
            local stream = ...
            local addr = stream:peer_addr()
            print("connected "..addr)
            stream:write("hello")
            
            local data = stream:read(100)
            data = data:match("^%s*(.-)%s*$") -- trim
            print("["..peer_addr.."] "..data)
            stream:close()
            return data == "test"
            "#,
        ).into_function().unwrap();
        
        assert!(func.call_async::<_, bool>(
            LuaTcpStream::new(TcpStream::connect("1.1.1.1:53").await.unwrap())
        ).await.unwrap());
    
    });
    Ok(())
}

