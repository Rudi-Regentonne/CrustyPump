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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_table_basic() {
        let html = r#"<table>
            <tr><td>1</td><td>Aussentemperatur</td><td>23.5</td><td>°C</td></tr>
            <tr><td>2</td><td>Druck</td><td>12.3</td><td>bar</td></tr>
        </table>"#;
        let rows = parse_table(html);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "Aussentemperatur");
        assert_eq!(rows[0].value, "23.5");
        assert_eq!(rows[0].unit, "°C");
        assert_eq!(rows[1].name, "Druck");
        assert_eq!(rows[1].value, "12.3");
        assert_eq!(rows[1].unit, "bar");
    }

    #[test]
    fn parse_table_no_rows() {
        let html = "<table></table>";
        let rows = parse_table(html);
        assert!(rows.is_empty());
    }

    #[test]
    fn parse_table_strips_inner_html() {
        let html = r#"<table>
            <tr><td>1</td><td>Vorlauf <b>heiss</b></td><td><span class="val">35.0</span></td><td>°C</td></tr>
        </table>"#;
        let rows = parse_table(html);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "Vorlauf heiss");
        assert_eq!(rows[0].value, "35.0");
    }

    fn make_menu_item(edesc: Option<&str>, name: &str, children: Vec<MenuItem>) -> MenuItem {
        MenuItem {
            edesc: edesc.map(|s| s.to_string()),
            name: name.to_string(),
            value: None,
            items: if children.is_empty() { None } else { Some(children) },
        }
    }

    #[test]
    fn find_by_edesc_finds_root() {
        let item = make_menu_item(Some("_INFORMATIONS"), "Info", vec![]);
        let found = item.find_by_edesc("_INFORMATIONS");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Info");
    }

    #[test]
    fn find_by_edesc_finds_nested() {
        let child = make_menu_item(Some("_INPUTS_OUTPUTS_INFO"), "IO", vec![]);
        let parent = make_menu_item(Some("_INFORMATIONS"), "Info", vec![child]);
        let found = parent.find_by_edesc("_INPUTS_OUTPUTS_INFO");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "IO");
    }

    #[test]
    fn find_by_edesc_missing_returns_none() {
        let item = make_menu_item(Some("_OTHER"), "Other", vec![]);
        let found = item.find_by_edesc("_MISSING");
        assert!(found.is_none());
    }

    #[test]
    fn find_inputs_outputs_info_root_finds_root() {
        let root = make_menu_item(Some("_INFORMATIONS"), "Info", vec![]);
        let items = [root];
        let found = find_inputs_outputs_info_root(&items);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Info");
    }

    #[test]
    fn find_inputs_outputs_info_root_missing() {
        let item = make_menu_item(Some("_OTHER"), "Other", vec![]);
        let items = [item];
        let found = find_inputs_outputs_info_root(&items);
        assert!(found.is_none());
    }
}
