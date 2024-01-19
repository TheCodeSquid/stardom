use std::io::{self, Write};

use crate::api::Api;

pub fn include<W: Write>(mut w: W, api: &Api) -> io::Result<()> {
    writeln!(w, "// This file was generated by stardom-codegen, so it probably shouldn't be edited manually\n")

    writeln!(w, "const ELEMENTS: &[&str] = &[")?;
    for element in &api.elements {
        writeln!(w, "    \"{}\",", element)?;
    }
    writeln!(w, "];\n")?;

    writeln!(w, "const ATTRS: &[&str] = &[")?;
    for attr in &api.attributes {
        writeln!(w, "    \"{}\",", attr)?;
    }
    writeln!(w, "];\n")?;

    writeln!(w, "const EVENTS: &[(&str, &str)] = &[")?;
    for (interface, names) in &api.events {
        for name in names {
            writeln!(w, "    (\"{}\", \"{}\"),", name, interface)?;
        }
    }
    writeln!(w, "];\n")?;

    writeln!(w, "define_tagged! {{")?;
    for element in &api.elements {
        writeln!(w, "    {},", element)?;
    }
    writeln!(w, "}}")?;

    Ok(())
}
