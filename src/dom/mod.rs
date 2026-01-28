pub mod document;
pub mod node;
pub mod parser;

pub use document::Document;
pub use node::{Node, NodeData, NodeId};
pub use parser::parse_html;
