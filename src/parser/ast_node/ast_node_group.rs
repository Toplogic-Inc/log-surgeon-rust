use crate::parser::ast_node::ast_node::AstNode;

#[derive(Debug)]
pub(crate) struct AstNodeGroup {
    m_op1: Box<AstNode>,
}

impl PartialEq for AstNodeGroup {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}
