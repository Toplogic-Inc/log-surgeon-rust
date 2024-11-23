use crate::parser::ast_node::ast_node::AstNode;

pub(crate) struct AstNodeStar {
    m_op1: Box<AstNode>,
}

impl AstNodeStar {
    pub(crate) fn new(p0: AstNode) -> AstNodeStar {
        AstNodeStar {
            m_op1: Box::new(p0),
        }
    }

    pub(crate) fn get_op1(&self) -> &AstNode {
        &self.m_op1
    }
}

impl PartialEq for AstNodeStar {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}

impl std::fmt::Debug for AstNodeStar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Star( {:?} )", self.m_op1)
    }
}
