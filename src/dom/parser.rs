use super::document::Document;
use super::node::NodeId;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData as RcNodeData, RcDom};

pub fn parse_html(html: &str) -> Document {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap();

    let mut document = Document::new();
    let root_id = document.root;
    convert_node(&dom.document, &mut document, root_id);
    document
}

fn convert_node(handle: &Handle, document: &mut Document, parent_id: NodeId) {
    let node = &*handle;

    match &node.data {
        RcNodeData::Document => {
            for child in node.children.borrow().iter() {
                convert_node(child, document, parent_id);
            }
        }
        RcNodeData::Element { name, attrs, .. } => {
            let tag_name = name.local.to_string();
            let element_id = document.create_element(&tag_name);

            for attr in attrs.borrow().iter() {
                let attr_name = attr.name.local.to_string();
                let attr_value = attr.value.to_string();
                document.set_attribute(element_id, &attr_name, &attr_value);
            }

            document.append_child(parent_id, element_id);

            for child in node.children.borrow().iter() {
                convert_node(child, document, element_id);
            }
        }
        RcNodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            if !text.trim().is_empty() {
                let text_id = document.create_text(&text);
                document.append_child(parent_id, text_id);
            }
        }
        RcNodeData::Comment { contents } => {
            let comment_id = document.create_comment(&contents.to_string());
            document.append_child(parent_id, comment_id);
        }
        RcNodeData::Doctype { .. } | RcNodeData::ProcessingInstruction { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = "<html><body><p>Hello World</p></body></html>";
        let doc = parse_html(html);

        let body = doc.get_body().expect("Should have body");
        let body_children = doc.children(body);
        assert!(!body_children.is_empty());

        let p_elements = doc.get_elements_by_tag_name("p");
        assert_eq!(p_elements.len(), 1);

        let text = doc.get_text_content(p_elements[0]);
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_parse_with_attributes() {
        let html = r#"<div id="main" class="container large">Content</div>"#;
        let doc = parse_html(html);

        let main = doc.get_element_by_id("main").expect("Should find #main");
        let node = doc.get_node(main).unwrap();
        let elem = node.as_element().unwrap();

        assert_eq!(elem.id(), Some("main"));
        assert!(elem.classes().iter().any(|c| c == "container"));
        assert!(elem.classes().iter().any(|c| c == "large"));
    }

    #[test]
    fn test_parse_nested_elements() {
        let html = "<div><span><a href=\"#\">Link</a></span></div>";
        let doc = parse_html(html);

        let divs = doc.get_elements_by_tag_name("div");
        assert_eq!(divs.len(), 1);

        let spans = doc.get_elements_by_tag_name("span");
        assert_eq!(spans.len(), 1);

        let links = doc.get_elements_by_tag_name("a");
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn test_parse_multiple_siblings() {
        let html = "<ul><li>One</li><li>Two</li><li>Three</li></ul>";
        let doc = parse_html(html);

        let li_elements = doc.get_elements_by_tag_name("li");
        assert_eq!(li_elements.len(), 3);
    }

    #[test]
    fn test_get_head_element() {
        let html = "<html><head><title>Test</title></head><body></body></html>";
        let doc = parse_html(html);

        let head = doc.get_head();
        assert!(head.is_some());

        let titles = doc.get_elements_by_tag_name("title");
        assert_eq!(titles.len(), 1);
    }

    #[test]
    fn test_parse_empty_elements() {
        let html = "<div><br><hr><img src=\"test.png\"></div>";
        let doc = parse_html(html);

        let brs = doc.get_elements_by_tag_name("br");
        assert_eq!(brs.len(), 1);

        let hrs = doc.get_elements_by_tag_name("hr");
        assert_eq!(hrs.len(), 1);

        let imgs = doc.get_elements_by_tag_name("img");
        assert_eq!(imgs.len(), 1);
    }

    #[test]
    fn test_parse_text_content() {
        let html = "<p>Hello <strong>bold</strong> world</p>";
        let doc = parse_html(html);

        let p = doc.get_elements_by_tag_name("p")[0];
        let text = doc.get_text_content(p);

        assert!(text.contains("Hello"));
        assert!(text.contains("bold"));
        assert!(text.contains("world"));
    }

    #[test]
    fn test_parse_link_href() {
        let html = r#"<a href="https://example.com">Link</a>"#;
        let doc = parse_html(html);

        let links = doc.get_elements_by_tag_name("a");
        assert_eq!(links.len(), 1);

        let node = doc.get_node(links[0]).unwrap();
        let elem = node.as_element().unwrap();
        assert_eq!(elem.get_attribute("href"), Some("https://example.com"));
    }

    #[test]
    fn test_parse_image_attributes() {
        let html = r#"<img src="image.png" alt="Description" width="100" height="50">"#;
        let doc = parse_html(html);

        let imgs = doc.get_elements_by_tag_name("img");
        assert_eq!(imgs.len(), 1);

        let node = doc.get_node(imgs[0]).unwrap();
        let elem = node.as_element().unwrap();
        assert_eq!(elem.get_attribute("src"), Some("image.png"));
        assert_eq!(elem.get_attribute("alt"), Some("Description"));
        assert_eq!(elem.get_attribute("width"), Some("100"));
        assert_eq!(elem.get_attribute("height"), Some("50"));
    }

    #[test]
    fn test_parse_multiple_classes() {
        let html = r#"<div class="class1 class2 class3">Content</div>"#;
        let doc = parse_html(html);

        let divs = doc.get_elements_by_tag_name("div");
        let node = doc.get_node(divs[0]).unwrap();
        let elem = node.as_element().unwrap();

        assert!(elem.classes().iter().any(|c| c == "class1"));
        assert!(elem.classes().iter().any(|c| c == "class2"));
        assert!(elem.classes().iter().any(|c| c == "class3"));
    }

    #[test]
    fn test_get_elements_by_class_name() {
        let html = r#"<div class="container"><span class="container">Text</span></div>"#;
        let doc = parse_html(html);

        let containers = doc.get_elements_by_class_name("container");
        assert_eq!(containers.len(), 2);
    }

    #[test]
    fn test_parse_style_attribute() {
        let html = r#"<div style="color: red; font-size: 16px;">Styled</div>"#;
        let doc = parse_html(html);

        let divs = doc.get_elements_by_tag_name("div");
        let node = doc.get_node(divs[0]).unwrap();
        let elem = node.as_element().unwrap();
        assert!(elem.get_attribute("style").is_some());
    }

    #[test]
    fn test_parse_data_attributes() {
        let html = r#"<div data-id="123" data-name="test">Data</div>"#;
        let doc = parse_html(html);

        let divs = doc.get_elements_by_tag_name("div");
        let node = doc.get_node(divs[0]).unwrap();
        let elem = node.as_element().unwrap();
        assert_eq!(elem.get_attribute("data-id"), Some("123"));
        assert_eq!(elem.get_attribute("data-name"), Some("test"));
    }

    #[test]
    fn test_parse_form_elements() {
        let html = r#"<form action="/submit"><input type="text" name="username"><button type="submit">Submit</button></form>"#;
        let doc = parse_html(html);

        let forms = doc.get_elements_by_tag_name("form");
        assert_eq!(forms.len(), 1);

        let inputs = doc.get_elements_by_tag_name("input");
        assert_eq!(inputs.len(), 1);

        let buttons = doc.get_elements_by_tag_name("button");
        assert_eq!(buttons.len(), 1);
    }

    #[test]
    fn test_parse_table_structure() {
        let html = "<table><thead><tr><th>Header</th></tr></thead><tbody><tr><td>Cell</td></tr></tbody></table>";
        let doc = parse_html(html);

        assert_eq!(doc.get_elements_by_tag_name("table").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("thead").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("tbody").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("tr").len(), 2);
        assert_eq!(doc.get_elements_by_tag_name("th").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("td").len(), 1);
    }

    #[test]
    fn test_node_count() {
        let html = "<div><p>Text</p></div>";
        let doc = parse_html(html);

        assert!(doc.node_count() > 0);
    }

    #[test]
    fn test_parent_child_relationship() {
        let html = "<div><p>Text</p></div>";
        let doc = parse_html(html);

        let divs = doc.get_elements_by_tag_name("div");
        let ps = doc.get_elements_by_tag_name("p");

        // Check that p is a child of div
        let div_children = doc.children(divs[0]);
        assert!(div_children.contains(&ps[0]));

        // Check that div is the parent of p
        let p_parent = doc.parent(ps[0]);
        assert_eq!(p_parent, Some(divs[0]));
    }

    #[test]
    fn test_parse_semantic_elements() {
        let html = "<article><header><h1>Title</h1></header><section><p>Content</p></section><footer>Footer</footer></article>";
        let doc = parse_html(html);

        assert_eq!(doc.get_elements_by_tag_name("article").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("header").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("section").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("footer").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("h1").len(), 1);
    }

    #[test]
    fn test_iter_nodes() {
        let html = "<div><span>Text</span></div>";
        let doc = parse_html(html);

        let count = doc.iter_nodes().count();
        assert!(count > 0);
    }

    #[test]
    fn test_parse_special_characters() {
        let html = "<p>&lt;script&gt; &amp; &quot;quotes&quot;</p>";
        let doc = parse_html(html);

        let text = doc.get_text_content(doc.get_elements_by_tag_name("p")[0]);
        assert!(text.contains("<script>"));
        assert!(text.contains("&"));
        assert!(text.contains("\"quotes\""));
    }

    #[test]
    fn test_case_insensitive_tags() {
        let html = "<DIV><P>Text</P></DIV>";
        let doc = parse_html(html);

        // Tags should be normalized to lowercase
        assert_eq!(doc.get_elements_by_tag_name("div").len(), 1);
        assert_eq!(doc.get_elements_by_tag_name("p").len(), 1);
    }

    #[test]
    fn test_set_text_content() {
        let html = "<p>Original</p>";
        let mut doc = parse_html(html);

        let p = doc.get_elements_by_tag_name("p")[0];
        doc.set_text_content(p, "New content");

        let text = doc.get_text_content(p);
        assert!(text.contains("New content"));
    }
}
