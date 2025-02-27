use handlebars::{Helper, Handlebars, RenderContext, RenderError};
use serde_json::value::Value;

pub fn len(h: &Helper,
           _: &Handlebars,
           rc: &mut RenderContext)
           -> Result<(), RenderError> {
    let param = h.param(0).unwrap();
    let length = match param.value() {
        &Value::Array(ref a) => a.len(),
        _ => 0,
    };

    let rendered = format!("{}", length);
    let _ = rc.writer.write(rendered.into_bytes().as_ref());
    Ok(())
}
