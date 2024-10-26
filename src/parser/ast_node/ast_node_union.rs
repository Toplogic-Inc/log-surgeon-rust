use crate::parser::ast_node::ast_node::AstNode;

pub(crate) struct AstNodeUnion {
    m_op1: Box<AstNode>,
    m_op2: Box<AstNode>,
}

impl AstNodeUnion {
    pub(crate) fn new(p0: AstNode, p1: AstNode) -> AstNodeUnion {
        AstNodeUnion {
            m_op1: Box::new(p0),
            m_op2: Box::new(p1),
        }
    }
}

impl PartialEq for AstNodeUnion {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1 && self.m_op2 == other.m_op2
    }
}

impl std::fmt::Debug for AstNodeUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Union( {:?} {:?} )", self.m_op1, self.m_op2)
    }
}
