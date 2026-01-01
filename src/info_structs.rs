use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct MenuItem {
    pub edesc: Option<String>,
    pub name: String,

    pub value: Option<Value>,
    pub items: Option<Vec<MenuItem>>,
}

impl MenuItem {
    pub fn find_by_edesc<'a>(&'a self, edesc: &str) -> Option<&'a MenuItem> {
        if let Some(ref e) = self.edesc {
            if e == edesc {
                return Some(self);
            }
        }
        if let Some(ref children) = self.items {
            for child in children {
                if let Some(found) = child.find_by_edesc(edesc) {
                    return Some(found);
                }
            }
        }
        None
    }
}
pub fn find_inputs_outputs_info_root(items: &[MenuItem]) -> Option<&MenuItem> {
    for item in items {
        if let Some(found) = item.find_by_edesc("_INFORMATIONS") {
            return Some(found);
        }
    }
    None
}

use regex::Regex;

#[derive(Debug)]
pub struct CellRow {
    pub name: String,
    pub value: String,
    pub unit: String,
}

//removes simple html tags
fn strip_tags(s: &str) -> String {
    let tag_re = Regex::new(r"(?s)<[^>]*>").unwrap();
    tag_re.replace_all(s, "").trim().to_string()
}

pub fn parse_table(html: &str) -> Vec<CellRow> {
    let row_re = Regex::new(
        r"(?si)<tr[^>]*>\s*<td[^>]*>\s*(?P<id>.*?)\s*</td>\s*<td[^>]*>\s*(?P<name>.*?)\s*</td>\s*<td[^>]*>\s*(?P<value>.*?)\s*</td>\s*<td[^>]*>\s*(?P<unit>.*?)\s*</td>\s*</tr>"
    ).unwrap();

    let mut out = Vec::new();
    for caps in row_re.captures_iter(html) {
        let name = strip_tags(&caps["name"]);
        let value = strip_tags(&caps["value"]);
        let unit = strip_tags(&caps["unit"]);
        out.push(CellRow { name, value, unit });
    }
    out
}
