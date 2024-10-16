// #[derive(Debug)]
pub(crate) enum ASTNode {
    Literal(char),                      // Single character literal
    Concat(Box<ASTNode>, Box<ASTNode>), // Concatenation of two expressions
    Union(Box<ASTNode>, Box<ASTNode>),  // Union of two expressions
    Star(Box<ASTNode>),                 // Kleene Star (zero or more)
    Plus(Box<ASTNode>),                 // One or more
    Optional(Box<ASTNode>),             // Zero or one (optional)
    Group(Box<ASTNode>),                // Capturing group
}

impl PartialEq for ASTNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ASTNode::Literal(l1), ASTNode::Literal(l2)) => l1 == l2,
            (ASTNode::Concat(l1, r1), ASTNode::Concat(l2, r2)) => l1 == l2 && r1 == r2,
            (ASTNode::Union(l1, r1), ASTNode::Union(l2, r2)) => l1 == l2 && r1 == r2,
            (ASTNode::Star(e1), ASTNode::Star(e2)) => e1 == e2,
            (ASTNode::Plus(e1), ASTNode::Plus(e2)) => e1 == e2,
            (ASTNode::Optional(e1), ASTNode::Optional(e2)) => e1 == e2,
            (ASTNode::Group(e1), ASTNode::Group(e2)) => e1 == e2,
            _ => false,
        }
    }
}

impl std::fmt::Debug for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTNode::Literal(c) => write!(f, "Literal({})", c),
            ASTNode::Concat(left, right) => write!(f, "Concat({:?}, {:?})", left, right),
            ASTNode::Union(left, right) => write!(f, "Union({:?}, {:?})", left, right),
            ASTNode::Star(node) => write!(f, "Star({:?})", node),
            ASTNode::Plus(node) => write!(f, "Plus({:?})", node),
            ASTNode::Optional(node) => write!(f, "Optional({:?})", node),
            ASTNode::Group(node) => write!(f, "Group({:?})", node),
        }
    }
}
