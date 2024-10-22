// #[derive(Debug)]
use super::ast_node_concat::ASTNodeConcat;
use super::ast_node_group::ASTNodeGroup;
use super::ast_node_literal::ASTNodeLiteral;
use super::ast_node_optional::ASTNodeOptional;
use super::ast_node_plus::ASTNodePlus;
use super::ast_node_star::ASTNodeStar;
use super::ast_node_union::ASTNodeUnion;

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
