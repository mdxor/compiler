use std::collections::HashMap;

pub struct MDXAST {
    pub head: String,
    pub nodes: Vec<MDXNode>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MDXNode {
    pub node_type: String,
    pub children: Vec<MDXNode>,
    pub attrs: HashMap<String, String>,
    pub text: String,
}

impl MDXNode {
    pub fn new(node_type: &str, text: &str) -> Self {
        MDXNode {
            node_type: node_type.to_string(),
            children: Vec::new(),
            attrs: HashMap::new(),
            text: text.to_string(),
        }
    }
    pub fn push(&mut self, child: MDXNode) {
        self.children.push(child);
    }
}
