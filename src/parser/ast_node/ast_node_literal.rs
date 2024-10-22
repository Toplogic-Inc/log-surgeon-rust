#[derive(Debug)]
pub(crate) struct ASTNodeLiteral {
    m_value: char,
}

impl PartialEq for ASTNodeLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.m_value == other.m_value
    }
}
