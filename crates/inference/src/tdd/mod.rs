use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_allocator::vec::Vec;
use mago_oracle::assertion::Assertion;
use mago_oracle::var::Var;

/// A literal in a decision diagram: a single [`Assertion`] about one variable.
///
/// Two literals are the same proposition only when both the variable and the
/// assertion match, so `($a === null)` and `($b === null)` are distinct.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Literal<'arena> {
    pub variable: Var<'arena>,
    pub assertion: Assertion<'arena>,
}

/// A handle to a node in a [`DecisionDiagram`]. [`Node::FALSE`] and
/// [`Node::TRUE`] are the two terminals; every other node tests one literal.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Node(u32);

impl Node {
    pub const FALSE: Node = Node(0);
    pub const TRUE: Node = Node(1);

    #[must_use]
    pub fn is_false(self) -> bool {
        self == Node::FALSE
    }

    #[must_use]
    pub fn is_true(self) -> bool {
        self == Node::TRUE
    }

    #[must_use]
    pub fn is_terminal(self) -> bool {
        self.0 < 2
    }
}

#[derive(Clone, Copy)]
struct Branch<'arena> {
    literal: Literal<'arena>,
    low: Node,
    high: Node,
}

const AND: u8 = 0;
const OR: u8 = 1;

/// A reduced, ordered binary decision diagram over [`Literal`]s.
///
/// Nodes are hash-consed and ordered by literal, so the structure is canonical:
/// a contradiction is exactly [`Node::FALSE`] and a tautology exactly
/// [`Node::TRUE`], with no saturation pass. `and`/`or`/`not`/`restrict` run a
/// memoized `apply`.
pub struct DecisionDiagram<'storage, 'arena, A: Arena> {
    branches: Vec<'storage, Branch<'arena>, A>,
    unique: HashMap<'storage, (Literal<'arena>, Node, Node), Node, A>,
    apply_memo: HashMap<'storage, (u8, Node, Node), Node, A>,
    negation_memo: HashMap<'storage, Node, Node, A>,
}

impl<'storage, 'arena, A: Arena> DecisionDiagram<'storage, 'arena, A> {
    #[must_use]
    pub fn new_in(arena: &'storage A) -> Self {
        Self {
            branches: Vec::new_in(arena),
            unique: HashMap::new_in(arena),
            apply_memo: HashMap::new_in(arena),
            negation_memo: HashMap::new_in(arena),
        }
    }

    /// The diagram for a single literal: `true` when the literal holds.
    #[must_use]
    pub fn literal(&mut self, literal: Literal<'arena>) -> Node {
        self.make_node(literal, Node::FALSE, Node::TRUE)
    }

    #[must_use]
    pub fn and(&mut self, left: Node, right: Node) -> Node {
        self.apply(AND, left, right)
    }

    #[must_use]
    pub fn or(&mut self, left: Node, right: Node) -> Node {
        self.apply(OR, left, right)
    }

    #[must_use]
    pub fn not(&mut self, node: Node) -> Node {
        if node.is_false() {
            return Node::TRUE;
        }
        if node.is_true() {
            return Node::FALSE;
        }
        if let Some(negation) = self.negation_memo.get(&node) {
            return *negation;
        }

        let branch = self.branch(node);
        let low = self.not(branch.low);
        let high = self.not(branch.high);
        let negation = self.make_node(branch.literal, low, high);
        self.negation_memo.insert(node, negation);

        negation
    }

    /// Cofactor `node` by fixing `literal` to `value`.
    #[must_use]
    pub fn restrict(&mut self, node: Node, literal: Literal<'arena>, value: bool) -> Node {
        if node.is_terminal() {
            return node;
        }

        let branch = self.branch(node);
        if branch.literal == literal {
            return if value { branch.high } else { branch.low };
        }
        if branch.literal > literal {
            return node;
        }

        let low = self.restrict(branch.low, literal, value);
        let high = self.restrict(branch.high, literal, value);

        self.make_node(branch.literal, low, high)
    }

    fn branch(&self, node: Node) -> Branch<'arena> {
        self.branches[node.0 as usize - 2]
    }

    fn make_node(&mut self, literal: Literal<'arena>, low: Node, high: Node) -> Node {
        if low == high {
            return low;
        }
        if let Some(node) = self.unique.get(&(literal, low, high)) {
            return *node;
        }

        let node = Node(self.branches.len() as u32 + 2);
        self.branches.push(Branch { literal, low, high });
        self.unique.insert((literal, low, high), node);

        node
    }

    fn apply(&mut self, operator: u8, left: Node, right: Node) -> Node {
        match operator {
            AND => {
                if left.is_false() || right.is_false() {
                    return Node::FALSE;
                }
                if left.is_true() {
                    return right;
                }
                if right.is_true() || left == right {
                    return left;
                }
            }
            _ => {
                if left.is_true() || right.is_true() {
                    return Node::TRUE;
                }
                if left.is_false() {
                    return right;
                }
                if right.is_false() || left == right {
                    return left;
                }
            }
        }

        let (left, right) = if left.0 <= right.0 { (left, right) } else { (right, left) };
        if let Some(result) = self.apply_memo.get(&(operator, left, right)) {
            return *result;
        }

        let left_branch = self.branch(left);
        let right_branch = self.branch(right);
        let literal = left_branch.literal.min(right_branch.literal);

        let (left_low, left_high) =
            if left_branch.literal == literal { (left_branch.low, left_branch.high) } else { (left, left) };
        let (right_low, right_high) =
            if right_branch.literal == literal { (right_branch.low, right_branch.high) } else { (right, right) };

        let low = self.apply(operator, left_low, right_low);
        let high = self.apply(operator, left_high, right_high);
        let result = self.make_node(literal, low, high);
        self.apply_memo.insert((operator, left, right), result);

        result
    }
}
