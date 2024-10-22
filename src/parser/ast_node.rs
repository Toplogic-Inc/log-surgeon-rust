// #[derive(Debug)]

#[derive(Debug)]
pub(crate) struct ASTNodeLiteral {
    m_value: char,
}

impl PartialEq for ASTNodeLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.m_value == other.m_value
    }
}

#[derive(Debug)]
pub(crate) struct ASTNodeConcat {
    m_op1: Box<ASTNode>,
    m_op2: Box<ASTNode>,
}

impl PartialEq for ASTNodeConcat {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1 && self.m_op2 == other.m_op2
    }
}

#[derive(Debug)]
pub(crate) struct ASTNodeUnion {
    m_op1: Box<ASTNode>,
    m_op2: Box<ASTNode>,
}

impl PartialEq for ASTNodeUnion {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1 && self.m_op2 == other.m_op2
    }
}

#[derive(Debug)]
pub(crate) struct ASTNodeStar {
    m_op1: Box<ASTNode>,
}

impl PartialEq for ASTNodeStar {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}

#[derive(Debug)]
pub(crate) struct ASTNodePlus {
    m_op1: Box<ASTNode>,
}

impl PartialEq for ASTNodePlus {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}

#[derive(Debug)]
pub(crate) struct ASTNodeOptional {
    m_op1: Box<ASTNode>,
}

impl PartialEq for ASTNodeOptional {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}

#[derive(Debug)]
pub(crate) struct ASTNodeGroup {
    m_op1: Box<ASTNode>,
}

impl PartialEq for ASTNodeGroup {
    fn eq(&self, other: &Self) -> bool {
        self.m_op1 == other.m_op1
    }
}

pub(crate) enum ASTNode {
    Literal(ASTNodeLiteral),   // Single character literal
    Concat(ASTNodeConcat),     // Concatenation of two expressions
    Union(ASTNodeUnion),       // Union of two expressions
    Star(ASTNodeStar),         // Kleene Star (zero or more)
    Plus(ASTNodePlus),         // One or more
    Optional(ASTNodeOptional), // Zero or one (optional)
    Group(ASTNodeGroup),       // Capturing group
}

impl PartialEq for ASTNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ASTNode::Literal(l1), ASTNode::Literal(l2)) => l1 == l2,
            (ASTNode::Concat(c1), ASTNode::Concat(c2)) => c1 == c2,
            (ASTNode::Union(u1), ASTNode::Union(u2)) => u1 == u2,
            (ASTNode::Star(s1), ASTNode::Star(s2)) => s1 == s2,
            (ASTNode::Plus(p1), ASTNode::Plus(p2)) => p1 == p2,
            (ASTNode::Optional(o1), ASTNode::Optional(o2)) => o1 == o2,
            (ASTNode::Group(g1), ASTNode::Group(g2)) => g1 == g2,
            _ => false,
        }
    }
}

impl std::fmt::Debug for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Literal(l) => write!(f, "Literal({:?})", l),
            ASTNode::Concat(c) => write!(f, "Concat({:?})", c),
            ASTNode::Union(u) => write!(f, "Union({:?})", u),
            ASTNode::Star(s) => write!(f, "Star({:?})", s),
            ASTNode::Plus(p) => write!(f, "Plus({:?})", p),
            ASTNode::Optional(o) => write!(f, "Optional({:?})", o),
            ASTNode::Group(g) => write!(f, "Group({:?})", g),
        }
    }
}
