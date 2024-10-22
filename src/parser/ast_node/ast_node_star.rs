use crate::parser::ast_node::ast_node::ASTNode;

#[derive(Debug)]
pub(crate) struct ASTNodeStar {
    m_op1: Box<ASTNode>,
}

impl PartialEq for ASTNodeStar {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}
