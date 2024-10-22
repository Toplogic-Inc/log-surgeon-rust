use crate::parser::ast_node::ast_node::AstNode;

#[derive(Debug)]
pub(crate) struct AstNodeStar {
    m_op1: Box<AstNode>,
}

impl PartialEq for AstNodeStar {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}
