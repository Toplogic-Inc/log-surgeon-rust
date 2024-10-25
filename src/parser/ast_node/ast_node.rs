// #[derive(Debug)]
use super::ast_node_concat::AstNodeConcat;
use super::ast_node_group::AstNodeGroup;
use super::ast_node_literal::AstNodeLiteral;
use super::ast_node_optional::AstNodeOptional;
use super::ast_node_plus::AstNodePlus;
use super::ast_node_star::AstNodeStar;
use super::ast_node_union::AstNodeUnion;

pub(crate) enum AstNode {
    Literal(AstNodeLiteral),
    Concat(AstNodeConcat),
    Union(AstNodeUnion),
    Star(AstNodeStar),
    Plus(AstNodePlus),
    Optional(AstNodeOptional),
    Group(AstNodeGroup),
}

impl PartialEq for AstNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AstNode::Literal(lhs), AstNode::Literal(rhs)) => lhs == rhs,
            (AstNode::Concat(lhs), AstNode::Concat(rhs)) => lhs == rhs,
            (AstNode::Union(lhs), AstNode::Union(rhs)) => lhs == rhs,
            (AstNode::Star(lhs), AstNode::Star(rhs)) => lhs == rhs,
            (AstNode::Plus(lhs), AstNode::Plus(rhs)) => lhs == rhs,
            (AstNode::Optional(lhs), AstNode::Optional(rhs)) => lhs == rhs,
            (AstNode::Group(lhs), AstNode::Group(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl std::fmt::Debug for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstNode::Literal(ast_node) => write!(f, "{:?}", ast_node),
            AstNode::Concat(ast_node) => write!(f, "{:?}", ast_node),
            AstNode::Union(ast_node) => write!(f, "{:?}", ast_node),
            AstNode::Star(ast_node) => write!(f, "{:?}", ast_node),
            AstNode::Plus(ast_node) => write!(f, "{:?}", ast_node),
            AstNode::Optional(ast_node) => write!(f, "{:?}", ast_node),
            AstNode::Group(ast_node) => write!(f, "{:?}", ast_node),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_node_literal_equality() {
        let node1 = AstNode::Literal(AstNodeLiteral::new('a'));
        let node2 = AstNode::Literal(AstNodeLiteral::new('a'));
        assert_eq!(node1, node2);
    }

    #[test]
    fn ast_node_concat_equality() {
        let node1 = AstNode::Concat(AstNodeConcat::new(
            AstNode::Literal(AstNodeLiteral::new('a')),
            AstNode::Literal(AstNodeLiteral::new('b')),
        ));
        let node2 = AstNode::Concat(AstNodeConcat::new(
            AstNode::Literal(AstNodeLiteral::new('a')),
            AstNode::Literal(AstNodeLiteral::new('b')),
        ));
        assert_eq!(node1, node2);
    }

    #[test]
    fn ast_node_union_equality() {
        let node1 = AstNode::Union(AstNodeUnion::new(
            AstNode::Literal(AstNodeLiteral::new('a')),
            AstNode::Literal(AstNodeLiteral::new('b')),
        ));
        let node2 = AstNode::Union(AstNodeUnion::new(
            AstNode::Literal(AstNodeLiteral::new('a')),
            AstNode::Literal(AstNodeLiteral::new('b')),
        ));
        assert_eq!(node1, node2);
    }

    #[test]
    fn ast_node_star_equality() {
        let node1 = AstNode::Star(AstNodeStar::new(AstNode::Literal(AstNodeLiteral::new('a'))));
        let node2 = AstNode::Star(AstNodeStar::new(AstNode::Literal(AstNodeLiteral::new('a'))));
        assert_eq!(node1, node2);
    }

    #[test]
    fn ast_node_plus_equality() {
        let node1 = AstNode::Plus(AstNodePlus::new(AstNode::Literal(AstNodeLiteral::new('a'))));
        let node2 = AstNode::Plus(AstNodePlus::new(AstNode::Literal(AstNodeLiteral::new('a'))));
        assert_eq!(node1, node2);
    }

    #[test]
    fn ast_node_optional_equality() {
        let node1 = AstNode::Optional(AstNodeOptional::new(AstNode::Literal(AstNodeLiteral::new(
            'a',
        ))));
        let node2 = AstNode::Optional(AstNodeOptional::new(AstNode::Literal(AstNodeLiteral::new(
            'a',
        ))));
        assert_eq!(node1, node2);
    }

    #[test]
    fn ast_node_group_equality() {
        let node1 = AstNode::Group(AstNodeGroup::new(AstNode::Literal(AstNodeLiteral::new(
            'a',
        ))));
        let node2 = AstNode::Group(AstNodeGroup::new(AstNode::Literal(AstNodeLiteral::new(
            'a',
        ))));
        assert_eq!(node1, node2);
    }

    #[test]
    fn ast_node_basic_debug() {
        let node = AstNode::Concat(AstNodeConcat::new(
            AstNode::Star(AstNodeStar::new(AstNode::Union(AstNodeUnion::new(
                AstNode::Literal(AstNodeLiteral::new('a')),
                AstNode::Literal(AstNodeLiteral::new('b')),
            )))),
            AstNode::Optional(AstNodeOptional::new(AstNode::Group(AstNodeGroup::new(
                AstNode::Plus(AstNodePlus::new(AstNode::Literal(AstNodeLiteral::new('c')))),
            )))),
        ));
        assert_eq!(format!("{:?}", node), "Concat( Star( Union( Literal('a') Literal('b') ) ) Optional( Group( Plus ( Literal('c') ) ) ) )");
    }
}
