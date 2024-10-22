use crate::parser::ast_node::ast_node::ASTNode;

#[derive(Debug)]
pub(crate) struct ASTNodeOptional {
    m_op1: Box<ASTNode>,
}

impl PartialEq for ASTNodeOptional {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}
