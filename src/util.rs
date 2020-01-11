use std::str::FromStr;

use crate::errors::Error;

pub fn parse_node(s: &str) -> Result<treexml::Element, Error> {
    let doc = treexml::Document::parse(s.as_bytes())?;

    Ok(doc
        .root
        .ok_or_else(|| Error::NullError("Root is empty".into()))?)
}

pub fn eval_node_contents<T>(node: &treexml::Element) -> Option<T>
where
    T: FromStr,
{
    match node.text {
        Some(ref v) => v.parse::<T>().ok(),
        _ => None,
    }
}

pub fn any_text(node: &treexml::Element) -> Option<String> {
    if node.cdata.is_some() {
        return node.cdata.clone();
    }
    if node.text.is_some() {
        return node.text.clone();
    }
    None
}

pub fn trimmed_optional(e: &Option<String>) -> Option<String> {
    e.clone().map(|v| v.trim().into())
}
