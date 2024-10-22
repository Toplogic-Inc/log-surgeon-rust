#[derive(Debug)]
pub(crate) struct AstNodeLiteral {
    m_value: char,
}

impl PartialEq for AstNodeLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.m_value == other.m_value
    }
}
