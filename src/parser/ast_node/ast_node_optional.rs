use crate::parser::ast_node::ast_node::AstNode;

pub(crate) struct AstNodeOptional {
    m_op1: Box<AstNode>,
}

impl AstNodeOptional {
    pub(crate) fn new(p0: AstNode) -> AstNodeOptional {
        AstNodeOptional {
            m_op1: Box::new(p0),
        }
    }
}

impl PartialEq for AstNodeOptional {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}

impl std::fmt::Debug for AstNodeOptional {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Optional( {:?} )", self.m_op1)
    }
}
