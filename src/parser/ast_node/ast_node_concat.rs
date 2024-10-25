use crate::parser::ast_node::ast_node::AstNode;

pub(crate) struct AstNodeConcat {
    m_op1: Box<AstNode>,
    m_op2: Box<AstNode>,
}

impl AstNodeConcat {
    pub(crate) fn new(p0: AstNode, p1: AstNode) -> AstNodeConcat {
        AstNodeConcat {
            m_op1: Box::new(p0),
            m_op2: Box::new(p1),
        }
    }
}

impl PartialEq for AstNodeConcat {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1 && self.m_op2 == other.m_op2
    }
}

impl std::fmt::Debug for AstNodeConcat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Concat( {:?} {:?} )", self.m_op1, self.m_op2)
    }
}
