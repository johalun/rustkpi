extern crate kuchiki;

use kuchiki::traits::*;
use kuchiki::{ElementData, NodeDataRef, NodeRef};

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ElementInformation {
    pub element_name: String,
    pub element_content: String,
    pub path: String,
}

impl ElementInformation {
    fn new<T: ToOutput>(content: &T, path: &[String]) -> ElementInformation {
        ElementInformation {
            element_name: content.name(),
            element_content: content.output(),
            path: path.join("/"),
        }
    }

    fn from_path(path: &[String]) -> ElementInformation {
        ElementInformation {
            element_name: String::new(),
            element_content: String::new(),
            path: path.join("/"),
        }
    }
}

trait ToOutput {
    fn output(&self) -> String;
    fn name(&self) -> String;
}

impl ToOutput for NodeDataRef<ElementData> {
    fn output(&self) -> String {
        format!("<{0} {1}>{2}</{0}>",
                (*self).name.local,
                (*(*self).attributes.borrow()).map
                                              .iter()
                                              .map(|(k, v)| format!("\"{}\"=\"{}\"", k.local, v))
                                              .collect::<Vec<String>>()
                                              .join(" "),
                self.text_contents())
    }

    fn name(&self) -> String {
        format!("{}", (*self).name.local)
    }
}

impl ToOutput for NodeRef {
    fn output(&self) -> String {
        self.to_string()
    }

    fn name(&self) -> String {
        if let Some(s) = self.to_string().split(' ').next().unwrap().split('>').next() {
            if s.is_empty() {
                String::new()
            } else {
                s.chars().skip(1).collect()
            }
        } else {
            String::new()
        }
    }
}

/// Contains the kind of difference and some information.
#[derive(Debug, Clone, PartialEq)]
pub enum Difference {
    /// Different node types at the same place (text vs data for example).
    NodeType {
        elem: ElementInformation,
        opposite_elem: ElementInformation,
    },
    /// Different node names (`div` vs `ul` for example).
    NodeName {
        elem: ElementInformation,
        opposite_elem: ElementInformation,
    },
    /// Different attributes for two nodes.
    NodeAttributes {
        elem: ElementInformation,
        elem_attributes: HashMap<String, String>,
        opposite_elem: ElementInformation,
        opposite_elem_attributes: HashMap<String, String>,
    },
    /// Different text content for two nodes.
    NodeText {
        elem: ElementInformation,
        elem_text: String,
        opposite_elem: ElementInformation,
        opposite_elem_text: String,
    },
    /// If an element isn't present in one of the two sides.
    NotPresent {
        elem: Option<ElementInformation>,
        opposite_elem: Option<ElementInformation>,
    },
}

impl Difference {
    pub fn is_node_type(&self) -> bool {
        match *self {
            Difference::NodeType { .. } => true,
            _ => false,
        }
    }

    pub fn is_node_name(&self) -> bool {
        match *self {
            Difference::NodeName { .. } => true,
            _ => false,
        }
    }

    pub fn is_node_attributes(&self) -> bool {
        match *self {
            Difference::NodeAttributes { .. } => true,
            _ => false,
        }
    }

    pub fn is_node_text(&self) -> bool {
        match *self {
            Difference::NodeText { .. } => true,
            _ => false,
        }
    }

    pub fn is_not_present(&self) -> bool {
        match *self {
            Difference::NotPresent { .. } => true,
            _ => false,
        }
    }
}

impl ToString for Difference {
    fn to_string(&self) -> String {
        match *self {
            Difference::NodeType { ref elem, ref opposite_elem } => {
                format!("{} => [Types differ]: expected \"{}\", found \"{}\"",
                        elem.path, elem.element_name, opposite_elem.element_name)
            }
            Difference::NodeName { ref elem, ref opposite_elem } => {
                format!("{} => [Tags differ]: expected \"{}\", found \"{}\"",
                        elem.path, elem.element_name, opposite_elem.element_name)
            }
            Difference::NodeAttributes { ref elem,
                                         ref elem_attributes,
                                         ref opposite_elem_attributes,
                                         .. } => {
                format!("{} => [Attributes differ in \"{}\"]: expected \"{:?}\", found \"{:?}\"",
                        elem.path, elem.element_name, elem_attributes, opposite_elem_attributes)
            }
            Difference::NodeText { ref elem, ref elem_text, ref opposite_elem_text, .. } => {
                format!("{} => [Texts differ]: expected {:?}, found {:?}",
                        elem.path, elem_text, opposite_elem_text)
            }
            Difference::NotPresent { ref elem, ref opposite_elem } => {
                if let Some(ref elem) = *elem {
                    format!("{} => [One element is missing]: expected {:?}",
                            elem.path, elem.element_name)
                } else if let Some(ref elem) = *opposite_elem {
                    format!("{} => [Unexpected element \"{}\"]: found {:?}",
                            elem.path, elem.element_name, elem.element_content)
                } else {
                    unreachable!()
                }
            }
        }
    }
}

fn check_elements(elem1: &NodeDataRef<ElementData>,
                  elem2: &NodeDataRef<ElementData>,
                  path: &[String]) -> Option<Difference> {
    let e1: &ElementData = &*elem1;
    let e2: &ElementData = &*elem2;
    if e1.name != e2.name {
        Some(Difference::NodeName {
            elem: ElementInformation::new(elem1, path),
            opposite_elem: ElementInformation::new(elem2, path),
        })
    } else {
        let all_attributes_match =
            (*e1.attributes.borrow()).map.iter().any(|(k, v)| {
                (*e2.attributes.borrow()).map.get(k) != Some(v)
            });
        if all_attributes_match {
            Some(Difference::NodeAttributes {
                elem: ElementInformation::new(elem1, path),
                elem_attributes: {
                    let map = &(*e1.attributes.borrow()).map;
                    let mut result = HashMap::with_capacity(map.len());
                    for (k, v) in map {
                        result.insert(format!("{}", k.local), v.clone());
                    }
                    result
                },
                opposite_elem: ElementInformation::new(elem2, path),
                opposite_elem_attributes: {
                    let map = &(*e2.attributes.borrow()).map;
                    let mut result = HashMap::with_capacity(map.len());
                    for (k, v) in map {
                        result.insert(format!("{}", k.local), v.clone());
                    }
                    result
                },
            })
        } else {
            None
        }
    }
}

fn check_if_comment_or_empty_text(e: &NodeRef) -> bool {
    e.as_comment().is_none() &&
    if let Some(t) = e.as_text() {
        !t.borrow().trim().is_empty()
    } else {
        true
    }
}

fn go_through_tree(element1: &NodeRef, element2: &NodeRef,
                   path: &mut Vec<String>) -> Vec<Difference> {
    let mut differences = Vec::new();
    let mut pos: HashMap<String, usize> = HashMap::new();
    let mut it1 = element1.children().filter(|e| check_if_comment_or_empty_text(e));
    let mut it2 = element2.children().filter(|e| check_if_comment_or_empty_text(e));
    loop {
        let (element1, element2) = (it1.next(), it2.next());
        if let Some(diff) = match (&element1, &element2) {
            (&Some(ref element1), &Some(ref element2)) => {
                match (element1.clone().into_element_ref(), element2.clone().into_element_ref()) {
                    (Some(e1), Some(e2)) => check_elements(&e1, &e2, path),
                    (None, None) => {
                        match (element1.as_text(), element2.as_text()) {
                            (Some(t1), Some(t2)) => {
                                if t1 != t2 {
                                    Some(Difference::NodeText {
                                        elem: ElementInformation::from_path(path),
                                        elem_text: t1.borrow().clone(),
                                        opposite_elem: ElementInformation::from_path(path),
                                        opposite_elem_text: t2.borrow().clone(),
                                    })
                                } else {
                                    None
                                }
                            }
                            (None, None) => None,
                            _ => {
                                Some(Difference::NodeType {
                                    elem: ElementInformation::new(element1, path),
                                    opposite_elem: ElementInformation::new(element2, path),
                                })
                            }
                        }
                    }
                    _ => {
                        Some(Difference::NodeType {
                            elem: ElementInformation::new(element1, path),
                            opposite_elem: ElementInformation::new(element2, path),
                        })
                    }
                }
            }
            (&Some(ref elem1), &None) => {
                Some(Difference::NotPresent {
                    elem: Some(ElementInformation::new(elem1, path)),
                    opposite_elem: None,
                })
            }
            (&None, &Some(ref elem2)) => {
                Some(Difference::NotPresent {
                    elem: None,
                    opposite_elem: Some(ElementInformation::new(elem2, path)),
                })
            }
            (&None, &None) => break,
        } {
            // need to add parent content
            differences.push(diff);
            continue
        }
        let need_pop = if let Some(ref elem) = element1 {
            if let Some(elem) = elem.as_element() {
                let pos = pos.entry((*elem.name.local).to_owned()).or_insert(0);
                path.push(format!("{}[{}]", elem.name.local, *pos));
                *pos += 1;
                true
            } else {
                false
            }
        } else {
            false
        };
        differences.extend_from_slice(&go_through_tree(&element1.unwrap(),
                                                       &element2.unwrap(),
                                                       path));
        if need_pop {
            path.pop();
        }
    }
    differences
}

/// Take two html content strings in output, returns a `Vec` containing the differences (if any).
pub fn get_differences(content1: &str, content2: &str) -> Vec<Difference> {
    go_through_tree(&kuchiki::parse_html().one(content1), &kuchiki::parse_html().one(content2),
                    &mut vec![String::new()])
}

#[test]
fn basic_diff() {
    let original = "<div><foo></foo></div>";
    let other = "<div><p></p></div>";

    let differences = get_differences(original, other);
    assert_eq!(differences.len(), 1, "{:?}", differences);
    assert_eq!(differences[0].is_node_name(), true, "{:?}", differences[0]);
}

// Test if we stop correctly at first difference and don't go down.
#[test]
fn children_diff() {
    let original = "<div><foo><p></p></foo></div>";
    let other = "<div><p><t></t></p></div>";

    let differences = get_differences(original, other);
    assert_eq!(differences.len(), 1, "{:?}", differences);
    assert_eq!(differences[0].is_node_name(), true, "{:?}", differences[0]);
}

#[test]
fn check_attributes_order() {
    let original = "<div id=\"g\" class=\"foo\"><foo><p></p></foo></div>";
    let other = "<div class=\"foo\" id=\"g\"><foo><p></p></foo></div>";

    let differences = get_differences(original, other);
    assert_eq!(differences.len(), 0, "{:?}", differences);
}

#[test]
fn check_attributes_missing() {
    let original = "<div id=\"g\" class=\"foo\"><foo><p></p></foo></div>";
    let other = "<div class=\"foo\"><foo><p></p></foo></div>";

    let differences = get_differences(original, other);
    assert_eq!(differences.len(), 1, "{:?}", differences);
    assert_eq!(differences[0].is_node_attributes(), true, "{:?}", differences[0]);
    match differences[0] {
        Difference::NodeAttributes { ref elem_attributes,
                                     ref opposite_elem_attributes,
                                     .. } => {
            let mut attributes = HashMap::new();
            attributes.insert("class".to_owned(), "foo".to_owned());
            assert_eq!(attributes, *opposite_elem_attributes,
                       "{:?}/{:?}", opposite_elem_attributes, attributes);
            attributes.insert("id".to_owned(), "g".to_owned());
            assert_eq!(attributes, *elem_attributes,
                       "{:?}/{:?}", elem_attributes, attributes);
        }
        _ => unreachable!(),
    }
}

#[test]
fn check_child_below() {
    let original = "<div><foo></foo><a></a><b><c></c></b></div>";
    let other = "<div><foo></foo><a></a><b><c><d></d></c></b></div>";

    let differences = get_differences(original, other);
    assert_eq!(differences.len(), 1, "{:?}", differences);
    assert_eq!(differences[0].is_not_present(), true, "{:?}", differences[0]);
}

#[test]
fn test_path() {
    let original = "<div><foo></foo><a></a><b><c></c></b></div>";
    let other = "<div><foo></foo><a></a><b><c><d></d></c></b></div>";

    let differences = get_differences(original, other);
    assert_eq!(differences.len(), 1, "{:?}", differences);
    assert_eq!(differences[0].is_not_present(), true, "{:?}", differences[0]);
    match differences[0] {
        Difference::NotPresent { ref elem, ref opposite_elem } => {
            assert_eq!(elem.is_none(), true, "{:?}", elem);
            assert_eq!(opposite_elem.is_some(), true, "{:?}", opposite_elem);
            assert_eq!(*opposite_elem,
                       Some(ElementInformation {
                           element_name: "d".to_owned(),
                           element_content: "<d></d>".to_owned(),
                           path: "/html[0]/body[0]/div[0]/b[0]/c[0]".to_owned(),
                       }),
                       "{:?}", opposite_elem);
        }
        _ => unreachable!(),
    }
    assert_eq!(differences[0].is_not_present(), true, "{:?}", differences[0]);
}

#[test]
fn test_issue_6() {
    let a = "<!-- .basic -->\n<span class=\"why\">chunky bacon</span>\n\n<!-- .emphasis -->\n\
             <em>chunky bacon</em>\n\n<!-- .emphasis_with_role -->\n<em class=\"why\">chunky bacon\
             </em>\n\n<!-- .strong -->\n<strong>chunky bacon</strong>\n\n<!-- .strong_with_role \
             -->\n<strong class=\"why\">chunky bacon</strong>\n\n<!-- .monospaced -->\n<code>\
             hello world!</code>\n\n<!-- .monospaced_with_role -->\n<code class=\"why\">hello \
             world!</code>\n\n<!-- .superscript -->\n<sup>super</sup>chunky bacon\n\n<!-- \
             .superscript_with_role -->\n<sup class=\"why\">super</sup>chunky bacon\n\n<!-- \
             .subscript -->\n<sub>sub</sub>chunky bacon\n\n<!-- .subscript_with_role -->\n<sub \
             class=\"why\">sub</sub>chunky bacon\n\n<!-- .mark -->\n<mark>chunky bacon</mark>\n\n\
             <!-- .double -->\n“chunky bacon”\n\n<!-- .double_with_role -->\n<span class=\"why\">\
             “chunky bacon”</span>\n\n<!-- .single -->\n‘chunky bacon’\n\n<!-- .single_with_role \
             -->\n<span class=\"why\">‘chunky bacon’</span>\n\n<!-- .asciimath -->\n\\$sqrt(4) = 2\
             \\$\n\n<!-- .latexmath -->\n\\($C = \\alpha + \\beta Y^{\\gamma} + \\epsilon$\\)\n\n\
             <!-- .with_id -->\n<a id=\"why\"></a><em>chunky bacon</em>\n\n<!-- \
             .mixed_monospace_bold_italic -->\n<code><strong><em>monospace bold italic phrase</em>\
             </strong></code> and le<code><strong><em>tt</em></strong></code>ers\n";
    let b = "<mark></mark><div class=\"paragraph\"><p>chunky bacon</p></div><mark></mark>";
    let _ = get_differences(a, b);
}

#[test]
fn test_attributes() {
    let a = r#"<span class="toto"></span>"#;
    let b = r#"<span class="tata"></span>"#;
    let differences = get_differences(a, b);
    assert_eq!(differences.len(), 1);

    let a = r#"<span foo="a" class="toto"></span>"#;
    let b = r#"<span class="toto" foo="a"></span>"#;
    let differences = get_differences(a, b);
    assert_eq!(differences.len(), 0);
}

#[test]
fn test_path_logic() {
    use Difference::NodeText;

    let a = r#"<div>
    <img src="...">
    <div>
        <p>here will be changed content</p>
    </div>
</div>"#;
    let b = r#"<div>
    <img src="...">
    <div>
        <p>here will be changed</p>
    </div>
</div>"#;

    let differences = get_differences(a, b);
    assert_eq!(differences,
               vec![NodeText {
                        elem: ElementInformation {
                                  element_name: String::new(),
                                  element_content: String::new(),
                                  path: "/html[0]/body[0]/div[0]/div[0]/p[0]".to_owned(),
                              },
                        elem_text: "here will be changed content".to_owned(),
                        opposite_elem: ElementInformation {
                                           element_name: String::new(),
                                           element_content: String::new(),
                                           path: "/html[0]/body[0]/div[0]/div[0]/p[0]".to_owned(),
                                       },
                        opposite_elem_text: "here will be changed".to_owned(),
                    }]);
}

#[test]
fn test_path_logic2() {
    use Difference::NodeText;

    let a = r#"<div>
    <div></div>
    <div>
        <p>here will be changed content</p>
    </div>
</div>"#;
    let b = r#"<div>
    <div></div>
    <div>
        <p>here will be changed</p>
    </div>
</div>"#;

    let differences = get_differences(a, b);
    assert_eq!(differences,
               vec![NodeText {
                        elem: ElementInformation {
                                  element_name: String::new(),
                                  element_content: String::new(),
                                  path: "/html[0]/body[0]/div[0]/div[1]/p[0]".to_owned(),
                              },
                        elem_text: "here will be changed content".to_owned(),
                        opposite_elem: ElementInformation {
                                           element_name: String::new(),
                                           element_content: String::new(),
                                           path: "/html[0]/body[0]/div[0]/div[1]/p[0]".to_owned(),
                                       },
                        opposite_elem_text: "here will be changed".to_owned(),
                    }]);
}

#[test]
fn test_unicode() {
    let a = r#"<div><div></div>០</div<"#;
    let b = r#"<div><div></div>០</div>"#;
    let differences = get_differences(a, b);
    assert_eq!(differences.len(), 0);
}
