

pub enum MathContent {
    Text(String),
    Node(MathNode),
}


/// Represents a node in a math expression
pub struct MathNode {
    children: Vec<MathContent>,
}

