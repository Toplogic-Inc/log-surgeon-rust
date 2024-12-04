use std::fmt;

pub(crate) struct AstNodeLiteral {
    m_value: char,
}

impl AstNodeLiteral {
    pub(crate) fn new(p0: char) -> AstNodeLiteral {
        AstNodeLiteral { m_value: p0 }
    }

    pub(crate) fn get_value(&self) -> char {
        self.m_value
    }
}

impl PartialEq for AstNodeLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.m_value == other.m_value
    }
}

impl fmt::Debug for AstNodeLiteral {
    fn fmt(&self, p: &mut fmt::Formatter) -> fmt::Result {
        write!(p, "Literal({:?})", self.m_value)
    }
}
