use crate::parser::ast_node::ast_node::AstNode;

#[derive(Debug)]
pub(crate) struct AstNodeUnion {
    m_op1: Box<AstNode>,
    m_op2: Box<AstNode>,
}

impl PartialEq for AstNodeUnion {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1 && self.m_op2 == other.m_op2
    }
}
