use mlua::LuaSerdeExt;
use paxhtml::{OwnedAttribute, OwnedElement};

pub fn register(lua: &mlua::Lua) -> mlua::Result<()> {
    let table = lua.create_table()?;

    table.set(
        "element",
        lua.create_function(move |lua, name: String| build_element_function(lua, name, false))?,
    )?;

    table.set(
        "void_element",
        lua.create_function(move |lua, name: String| build_element_function(lua, name, true))?,
    )?;

    for name in paxhtml::builder::NON_VOID_TAGS {
        table.set(*name, build_element_function(lua, name.to_string(), false)?)?;
    }

    for name in paxhtml::builder::VOID_TAGS {
        table.set(*name, build_element_function(lua, name.to_string(), true)?)?;
    }

    table.set(
        "fragment",
        lua.create_function(move |lua, children: mlua::Value| {
            let children = process_children_value(lua, children)?;
            lua.to_value(&OwnedElement::Fragment { children })
        })?,
    )?;

    table.set(
        "text",
        lua.create_function(move |lua, text: String| lua.to_value(&OwnedElement::Text { text }))?,
    )?;

    table.set(
        "empty",
        lua.create_function(move |lua, _: ()| lua.to_value(&OwnedElement::Empty))?,
    )?;

    lua.globals().set("h", table)?;

    Ok(())
}

fn build_element_function(
    lua: &mlua::Lua,
    name: String,
    void: bool,
) -> mlua::Result<mlua::Function> {
    lua.create_function(move |lua, attributes: mlua::Table| {
        let name = name.clone();
        let attributes = attributes
            .pairs::<mlua::Value, mlua::String>()
            .map(|p| {
                let (key, value) = p?;
                let (key, value) = if key.is_integer() {
                    (value.to_string_lossy(), None)
                } else if let mlua::Value::String(key) = key {
                    (key.to_string_lossy(), Some(value.to_string_lossy()))
                } else {
                    return Err(mlua::Error::RuntimeError(
                        "Invalid attribute key".to_string(),
                    ));
                };
                Ok(OwnedAttribute { key, value })
            })
            .collect::<mlua::Result<Vec<OwnedAttribute>>>()?;

        if void {
            Ok(lua.to_value(&OwnedElement::Tag {
                name: name.clone(),
                attributes: attributes.clone(),
                children: vec![],
                void,
            })?)
        } else {
            Ok(mlua::Value::Function(lua.create_function(
                move |lua, children: mlua::Value| {
                    let children = process_children_value(lua, children)?;

                    lua.to_value(&OwnedElement::Tag {
                        name: name.clone(),
                        attributes: attributes.clone(),
                        children,
                        void,
                    })
                },
            )?))
        }
    })
}

fn process_children_value(
    lua: &mlua::Lua,
    children: mlua::Value,
) -> mlua::Result<Vec<OwnedElement>> {
    let children = if let mlua::Value::Table(table_children) = &children {
        if let Ok(element) = lua.from_value::<OwnedElement>(children.clone()) {
            return Ok(vec![element]);
        }

        let mut output = vec![];
        for v in table_children.sequence_values() {
            let v: mlua::Value = v?;
            if v.is_string() {
                output.push(OwnedElement::Text {
                    text: v.to_string()?,
                });
            } else if v.is_table() {
                output.extend(process_children_value(lua, v)?);
            } else {
                return Err(mlua::Error::RuntimeError(format!(
                    "Invalid child type (child): {}",
                    v.type_name()
                )));
            }
        }
        output
    } else if let mlua::Value::String(string) = children {
        vec![OwnedElement::Text {
            text: string.to_string_lossy(),
        }]
    } else {
        return Err(mlua::Error::RuntimeError(format!(
            "Invalid child type (children): {}",
            children.type_name()
        )));
    };
    Ok(children)
}
