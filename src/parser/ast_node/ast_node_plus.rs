use crate::parser::ast_node::ast_node::AstNode;

pub(crate) struct AstNodePlus {
    m_op1: Box<AstNode>,
}

impl AstNodePlus {
    pub(crate) fn new(p0: AstNode) -> AstNodePlus {
        AstNodePlus {
            m_op1: Box::new(p0),
        }
    }
}

impl PartialEq for AstNodePlus {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}

impl std::fmt::Debug for AstNodePlus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Plus ( {:?} )", self.m_op1)
    }
}
