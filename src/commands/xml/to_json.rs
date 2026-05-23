use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::core::error::{CliError, ErrorCode};
use crate::core::input::resolve_input;
use crate::core::output::Out;
use crate::core::tty::is_stdin_tty;

use super::RunArgs;

#[derive(Serialize)]
struct Out0 {
    json: String,
}

pub fn run(args: RunArgs, out: &Out) -> Result<(), CliError> {
    let input = resolve_input(
        args.input.as_deref(),
        args.in_path.as_deref(),
        is_stdin_tty(),
    )?;
    let value = parse_xml_to_value(input.as_str()?)?;
    let json = serde_json::to_string(&value).map_err(|e| {
        CliError::new(
            ErrorCode::Internal,
            format!("json serialization failed: {e}"),
        )
    })?;
    out.emit_value(&Out0 { json })
}

struct Frame {
    name: String,
    attrs: Map<String, Value>,
    children: Vec<(String, Value)>,
    text: String,
}

pub fn parse_xml_to_value(xml: &str) -> Result<Value, CliError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<Frame> = None;
    let mut buf = Vec::new();

    fn read_attrs(e: &quick_xml::events::BytesStart) -> Map<String, Value> {
        let mut attrs = Map::new();
        for attr in e.attributes().flatten() {
            let k = format!("@{}", String::from_utf8_lossy(attr.key.as_ref()));
            let v = attr
                .unescape_value()
                .map(|c| c.into_owned())
                .unwrap_or_default();
            attrs.insert(k, Value::String(v));
        }
        attrs
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(
                    CliError::new(ErrorCode::InvalidXml, format!("invalid XML: {e}"))
                        .with_hint("ensure the input is well-formed XML"),
                );
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                stack.push(Frame {
                    name,
                    attrs: read_attrs(&e),
                    children: Vec::new(),
                    text: String::new(),
                });
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let attrs = read_attrs(&e);
                let v = if attrs.is_empty() {
                    Value::String(String::new())
                } else {
                    Value::Object(attrs)
                };
                if let Some(top) = stack.last_mut() {
                    top.children.push((name, v));
                } else {
                    return Err(CliError::new(
                        ErrorCode::InvalidXml,
                        "XML has no root element wrapping content",
                    ));
                }
            }
            Ok(Event::Text(t)) => {
                if let Some(top) = stack.last_mut() {
                    let s = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                    if !s.is_empty() {
                        top.text.push_str(&s);
                    }
                }
            }
            Ok(Event::End(_)) => {
                let frame = match stack.pop() {
                    Some(f) => f,
                    None => {
                        return Err(CliError::new(
                            ErrorCode::InvalidXml,
                            "XML has an end tag without a matching start tag",
                        ));
                    }
                };
                let v = frame_to_value(&frame);
                if let Some(parent) = stack.last_mut() {
                    parent.children.push((frame.name, v));
                } else {
                    root = Some(frame);
                }
            }
            _ => {}
        }
        buf.clear();
    }

    if let Some(frame) = root {
        let v = frame_to_value(&frame);
        let mut wrapper = Map::new();
        wrapper.insert(frame.name, v);
        Ok(Value::Object(wrapper))
    } else {
        Err(CliError::new(
            ErrorCode::InvalidXml,
            "XML has no root element",
        ))
    }
}

fn frame_to_value(frame: &Frame) -> Value {
    if frame.attrs.is_empty() && frame.children.is_empty() {
        return Value::String(frame.text.clone());
    }
    let mut map = frame.attrs.clone();
    if !frame.text.is_empty() {
        map.insert("#text".to_string(), Value::String(frame.text.clone()));
    }
    use std::collections::BTreeMap;
    let mut grouped: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for (k, v) in &frame.children {
        if !grouped.contains_key(k) {
            order.push(k.clone());
        }
        grouped.entry(k.clone()).or_default().push(v.clone());
    }
    for k in order {
        let vs = grouped.remove(&k).unwrap();
        if vs.len() == 1 {
            map.insert(k, vs.into_iter().next().unwrap());
        } else {
            map.insert(k, Value::Array(vs));
        }
    }
    Value::Object(map)
}
