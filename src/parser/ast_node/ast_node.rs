// #[derive(Debug)]
use super::ast_node_concat::AstNodeConcat;
use super::ast_node_group::AstNodeGroup;
use super::ast_node_literal::AstNodeLiteral;
use super::ast_node_optional::AstNodeOptional;
use super::ast_node_plus::AstNodePlus;
use super::ast_node_star::AstNodeStar;
use super::ast_node_union::AstNodeUnion;

pub(crate) enum AstNode {
    Literal(AstNodeLiteral),   // Single character literal
    Concat(AstNodeConcat),     // Concatenation of two expressions
    Union(AstNodeUnion),       // Union of two expressions
    Star(AstNodeStar),         // Kleene Star (zero or more)
    Plus(AstNodePlus),         // One or more
    Optional(AstNodeOptional), // Zero or one (optional)
    Group(AstNodeGroup),       // Capturing group
}

impl PartialEq for AstNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AstNode::Literal(l1), AstNode::Literal(l2)) => l1 == l2,
            (AstNode::Concat(c1), AstNode::Concat(c2)) => c1 == c2,
            (AstNode::Union(u1), AstNode::Union(u2)) => u1 == u2,
            (AstNode::Star(s1), AstNode::Star(s2)) => s1 == s2,
            (AstNode::Plus(p1), AstNode::Plus(p2)) => p1 == p2,
            (AstNode::Optional(o1), AstNode::Optional(o2)) => o1 == o2,
            (AstNode::Group(g1), AstNode::Group(g2)) => g1 == g2,
            _ => false,
        }
    }
}

impl std::fmt::Debug for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstNode::Literal(l) => write!(f, "Literal({:?})", l),
            AstNode::Concat(c) => write!(f, "Concat({:?})", c),
            AstNode::Union(u) => write!(f, "Union({:?})", u),
            AstNode::Star(s) => write!(f, "Star({:?})", s),
            AstNode::Plus(p) => write!(f, "Plus({:?})", p),
            AstNode::Optional(o) => write!(f, "Optional({:?})", o),
            AstNode::Group(g) => write!(f, "Group({:?})", g),
        }
    }
}
