use super::interpreter::{Interpreter, JsObject, Value};
use crate::dom::{Document, NodeId};
use std::cell::RefCell;
use std::rc::Rc;

pub struct DomBindings {
    document: Rc<RefCell<Document>>,
}

impl DomBindings {
    pub fn new(document: Rc<RefCell<Document>>) -> Self {
        Self { document }
    }

    pub fn setup_globals(&self, interpreter: &mut Interpreter) {
        let document_obj = self.create_document_object();
        interpreter.global.borrow_mut().set("document".to_string(), document_obj);
    }

    fn create_document_object(&self) -> Value {
        let obj = Rc::new(RefCell::new(JsObject::new()));

        Value::Object(obj)
    }

    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.document.borrow().get_element_by_id(id)
    }

    pub fn get_inner_html(&self, node_id: NodeId) -> String {
        self.document.borrow().get_text_content(node_id)
    }

    pub fn set_inner_html(&self, node_id: NodeId, html: &str) {
        self.document.borrow_mut().set_text_content(node_id, html);
    }

    pub fn get_style_property(&self, _node_id: NodeId, _property: &str) -> Option<String> {
        None
    }

    pub fn set_style_property(&self, _node_id: NodeId, _property: &str, _value: &str) {
    }
}

pub fn create_element_object(node_id: NodeId, bindings: &DomBindings) -> Value {
    let obj = Rc::new(RefCell::new(JsObject::new()));

    obj.borrow_mut().set("nodeId".to_string(), Value::Number(node_id as f64));

    let inner_html = bindings.get_inner_html(node_id);
    obj.borrow_mut().set("innerHTML".to_string(), Value::String(inner_html));

    Value::Object(obj)
}
