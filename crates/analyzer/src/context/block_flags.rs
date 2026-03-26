#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BlockContextFlags(u32);

impl BlockContextFlags {
    pub const INSIDE_CONDITIONAL: u32 = 1 << 0;
    pub const INSIDE_ISSET: u32 = 1 << 1;
    pub const INSIDE_UNSET: u32 = 1 << 2;
    pub const INSIDE_GENERAL_USE: u32 = 1 << 3;
    pub const INSIDE_RETURN: u32 = 1 << 4;
    pub const INSIDE_THROW: u32 = 1 << 5;
    pub const INSIDE_ASSIGNMENT: u32 = 1 << 6;
    pub const INSIDE_ASSIGNMENT_OPERATION: u32 = 1 << 7;
    pub const INSIDE_LOOP: u32 = 1 << 8;
    pub const INSIDE_CALL: u32 = 1 << 9;
    pub const INSIDE_TRY: u32 = 1 << 10;
    pub const INSIDE_LOOP_EXPRESSIONS: u32 = 1 << 11;
    pub const INSIDE_NEGATION: u32 = 1 << 12;
    pub const INSIDE_VARIABLE_REFERENCE: u32 = 1 << 13;
    pub const HAS_RETURNED: u32 = 1 << 14;
    pub const COLLECT_INITIALIZATIONS: u32 = 1 << 15;
    pub const CALLS_PARENT_CONSTRUCTOR: u32 = 1 << 16;

    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn contains(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }

    #[inline]
    pub fn set(&mut self, flag: u32, value: bool) {
        let mask = (value as u32).wrapping_neg();

        self.0 = (self.0 & !flag) | (flag & mask);
    }

    #[inline]
    pub fn insert(&mut self, flag: u32) {
        self.0 |= flag;
    }

    #[inline]
    pub fn remove(&mut self, flag: u32) {
        self.0 &= !flag;
    }

    #[inline(always)]
    pub const fn inside_conditional(&self) -> bool {
        self.contains(Self::INSIDE_CONDITIONAL)
    }

    #[inline(always)]
    pub const fn inside_isset(&self) -> bool {
        self.contains(Self::INSIDE_ISSET)
    }

    #[inline(always)]
    pub const fn inside_unset(&self) -> bool {
        self.contains(Self::INSIDE_UNSET)
    }

    #[inline(always)]
    pub const fn inside_general_use(&self) -> bool {
        self.contains(Self::INSIDE_GENERAL_USE)
    }

    #[inline(always)]
    pub const fn inside_return(&self) -> bool {
        self.contains(Self::INSIDE_RETURN)
    }

    #[inline(always)]
    pub const fn inside_throw(&self) -> bool {
        self.contains(Self::INSIDE_THROW)
    }

    #[inline(always)]
    pub const fn inside_assignment(&self) -> bool {
        self.contains(Self::INSIDE_ASSIGNMENT)
    }

    #[inline(always)]
    pub const fn inside_assignment_operation(&self) -> bool {
        self.contains(Self::INSIDE_ASSIGNMENT_OPERATION)
    }

    #[inline(always)]
    pub const fn inside_loop(&self) -> bool {
        self.contains(Self::INSIDE_LOOP)
    }

    #[inline(always)]
    pub const fn inside_call(&self) -> bool {
        self.contains(Self::INSIDE_CALL)
    }

    #[inline(always)]
    pub const fn inside_try(&self) -> bool {
        self.contains(Self::INSIDE_TRY)
    }

    #[inline(always)]
    pub const fn inside_loop_expressions(&self) -> bool {
        self.contains(Self::INSIDE_LOOP_EXPRESSIONS)
    }

    #[inline(always)]
    pub const fn inside_negation(&self) -> bool {
        self.contains(Self::INSIDE_NEGATION)
    }

    #[inline(always)]
    pub const fn inside_variable_reference(&self) -> bool {
        self.contains(Self::INSIDE_VARIABLE_REFERENCE)
    }

    #[inline(always)]
    pub const fn has_returned(&self) -> bool {
        self.contains(Self::HAS_RETURNED)
    }

    #[inline(always)]
    pub const fn collect_initializations(&self) -> bool {
        self.contains(Self::COLLECT_INITIALIZATIONS)
    }

    #[inline(always)]
    pub const fn calls_parent_constructor(&self) -> bool {
        self.contains(Self::CALLS_PARENT_CONSTRUCTOR)
    }

    #[inline(always)]
    pub fn set_inside_conditional(&mut self, value: bool) {
        self.set(Self::INSIDE_CONDITIONAL, value);
    }

    #[inline(always)]
    pub fn set_inside_isset(&mut self, value: bool) {
        self.set(Self::INSIDE_ISSET, value);
    }

    #[inline(always)]
    pub fn set_inside_unset(&mut self, value: bool) {
        self.set(Self::INSIDE_UNSET, value);
    }

    #[inline(always)]
    pub fn set_inside_general_use(&mut self, value: bool) {
        self.set(Self::INSIDE_GENERAL_USE, value);
    }

    #[inline(always)]
    pub fn set_inside_return(&mut self, value: bool) {
        self.set(Self::INSIDE_RETURN, value);
    }

    #[inline(always)]
    pub fn set_inside_throw(&mut self, value: bool) {
        self.set(Self::INSIDE_THROW, value);
    }

    #[inline(always)]
    pub fn set_inside_assignment(&mut self, value: bool) {
        self.set(Self::INSIDE_ASSIGNMENT, value);
    }

    #[inline(always)]
    pub fn set_inside_assignment_operation(&mut self, value: bool) {
        self.set(Self::INSIDE_ASSIGNMENT_OPERATION, value);
    }

    #[inline(always)]
    pub fn set_inside_loop(&mut self, value: bool) {
        self.set(Self::INSIDE_LOOP, value);
    }

    #[inline(always)]
    pub fn set_inside_call(&mut self, value: bool) {
        self.set(Self::INSIDE_CALL, value);
    }

    #[inline(always)]
    pub fn set_inside_try(&mut self, value: bool) {
        self.set(Self::INSIDE_TRY, value);
    }

    #[inline(always)]
    pub fn set_inside_loop_expressions(&mut self, value: bool) {
        self.set(Self::INSIDE_LOOP_EXPRESSIONS, value);
    }

    #[inline(always)]
    pub fn set_inside_negation(&mut self, value: bool) {
        self.set(Self::INSIDE_NEGATION, value);
    }

    #[inline(always)]
    pub fn set_inside_variable_reference(&mut self, value: bool) {
        self.set(Self::INSIDE_VARIABLE_REFERENCE, value);
    }

    #[inline(always)]
    pub fn set_has_returned(&mut self, value: bool) {
        self.set(Self::HAS_RETURNED, value);
    }

    #[inline(always)]
    pub fn set_collect_initializations(&mut self, value: bool) {
        self.set(Self::COLLECT_INITIALIZATIONS, value);
    }

    #[inline(always)]
    pub fn set_calls_parent_constructor(&mut self, value: bool) {
        self.set(Self::CALLS_PARENT_CONSTRUCTOR, value);
    }
}
