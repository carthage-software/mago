use std::cmp::max;
use std::cmp::min;
use std::ops::Add;
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Rem;
use std::ops::Shl;
use std::ops::Shr;
use std::ops::Sub;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_atom::i64_atom;

use crate::ttype::TType;

/// Represents an integer type in a static analysis context, which can be either a
/// specific known value or a range of possible values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
pub enum TInteger {
    /// A specific, known integer value (e.g., `5`, `-100`).
    Literal(i64),
    /// A range from a specific value to positive infinity (e.g., `int<1, max>`).
    From(i64),
    /// A range from negative infinity to a specific value (e.g., `int<min, 100>`).
    To(i64),
    /// A bounded range between two values, inclusive (e.g., `int<1, 100>`).
    Range(i64, i64),
    /// The most general integer type, representing any possible integer value.
    Unspecified,
}

impl TInteger {
    /// Creates a new integer type from an optional literal value.
    ///
    /// If `Some(v)` is provided, it creates a `Literal(v)`. Otherwise, `Unspecified`.
    #[inline]
    pub const fn new(value: Option<i64>) -> Self {
        match value {
            Some(v) => TInteger::Literal(v),
            None => TInteger::Unspecified,
        }
    }

    /// Creates the most specific integer type possible from optional lower and upper bounds.
    #[inline]
    pub const fn from_bounds(from: Option<i64>, to: Option<i64>) -> Self {
        match (from, to) {
            (Some(f), Some(t)) => {
                if f == t {
                    TInteger::Literal(f)
                } else if f < t {
                    TInteger::Range(f, t)
                } else {
                    TInteger::Unspecified
                }
            }
            (Some(f), None) => TInteger::From(f),
            (None, Some(t)) => TInteger::To(t),
            (None, None) => TInteger::Unspecified,
        }
    }

    /// Creates an instance representing the general `int` type.
    #[inline]
    pub const fn unspecified() -> Self {
        Self::Unspecified
    }

    /// Creates an instance representing a literal integer type (e.g., `5`).
    #[inline]
    pub const fn literal(value: i64) -> Self {
        Self::Literal(value)
    }

    /// Creates an instance representing a `positive-int` (`int<1, max>`).
    #[inline]
    pub const fn positive() -> Self {
        Self::From(1)
    }

    /// Creates an instance representing a `non-negative-int` (`int<0, max>`).
    #[inline]
    pub const fn non_negative() -> Self {
        Self::From(0)
    }

    /// Creates an instance representing a `negative-int` (`int<min, -1>`).
    #[inline]
    pub const fn negative() -> Self {
        Self::To(-1)
    }

    /// Creates an instance representing a `non-positive-int` (`int<min, 0>`).
    #[inline]
    pub const fn non_positive() -> Self {
        Self::To(0)
    }

    /// Returns `true` if the type is `Unspecified`.
    #[inline]
    pub const fn is_unspecified(&self) -> bool {
        matches!(self, TInteger::Unspecified)
    }

    /// Returns `true` if the type is a `Literal`.
    #[inline]
    pub const fn is_literal(&self) -> bool {
        matches!(self, TInteger::Literal(_))
    }

    /// Returns `true` if the type is a `Range`.
    #[inline]
    pub const fn is_range(&self) -> bool {
        matches!(self, TInteger::Range(_, _))
    }

    /// Returns `true` if the type is a `From` range.
    #[inline]
    pub const fn is_from(&self) -> bool {
        matches!(self, TInteger::From(_))
    }

    /// Returns `true` if the type is a `To` range.
    #[inline]
    pub const fn is_to(&self) -> bool {
        matches!(self, TInteger::To(_))
    }

    /// Returns `true` if the type is exactly `Literal(0)`.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        matches!(self, TInteger::Literal(0))
    }

    /// Returns `true` if all possible values in this type are known to be positive (`> 0`).
    #[inline]
    pub const fn is_positive(&self) -> bool {
        match *self {
            TInteger::From(f) => f > 0,
            TInteger::Range(f, _) => f > 0,
            TInteger::Literal(l) => l > 0,
            _ => false,
        }
    }

    /// Returns `true` if all possible values in this type are known to be negative (`< 0`).
    #[inline]
    pub const fn is_negative(&self) -> bool {
        match *self {
            TInteger::To(t) => t < 0,
            TInteger::Range(_, t) => t < 0,
            TInteger::Literal(l) => l < 0,
            _ => false,
        }
    }

    /// Returns `true` if all possible values in this type are known to be non-negative (`>= 0`).
    #[inline]
    pub const fn is_non_negative(&self) -> bool {
        match *self {
            TInteger::From(f) => f >= 0,
            TInteger::Range(f, _) => f >= 0,
            TInteger::Literal(l) => l >= 0,
            _ => false,
        }
    }

    /// Returns `true` if all possible values in this type are known to be non-positive (`<= 0`).
    #[inline]
    pub const fn is_non_positive(&self) -> bool {
        match *self {
            TInteger::To(t) => t <= 0,
            TInteger::Range(_, t) => t <= 0,
            TInteger::Literal(l) => l <= 0,
            _ => false,
        }
    }

    /// Returns `true` if the type's possible set of values includes zero.
    #[inline]
    pub const fn can_be_zero(&self) -> bool {
        match *self {
            TInteger::Unspecified | TInteger::Literal(0) => true,
            TInteger::From(f) => f <= 0,
            TInteger::To(t) => t >= 0,
            TInteger::Range(f, t) => f <= 0 && t >= 0,
            _ => false,
        }
    }

    /// If the type is a `Literal`, returns its value. Otherwise, returns `None`.
    #[inline]
    pub const fn get_literal_value(&self) -> Option<i64> {
        match *self {
            TInteger::Literal(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the minimum possible value for the type, if a lower bound exists.
    #[inline]
    pub const fn get_minimum_value(&self) -> Option<i64> {
        match *self {
            TInteger::Literal(value) | TInteger::From(value) | TInteger::Range(value, _) => Some(value),
            _ => None,
        }
    }

    /// Returns the maximum possible value for the type, if an upper bound exists.
    #[inline]
    pub const fn get_maximum_value(&self) -> Option<i64> {
        match *self {
            TInteger::Literal(value) | TInteger::To(value) | TInteger::Range(_, value) => Some(value),
            _ => None,
        }
    }

    /// Checks if the integer type represented by `input` is fully contained
    /// within the integer type represented by `self`.
    ///
    /// This is useful for type checking, for example, to determine if a value
    /// of type `int<1, 5>` can be safely assigned to a variable of type `int<0, 10>`.
    ///
    /// # Rules
    ///
    /// - An `Unspecified` container can hold any other integer type.
    /// - A specific container (e.g., `Literal`, `Range`) cannot hold an `Unspecified` type.
    #[inline]
    pub const fn contains(&self, input: TInteger) -> bool {
        use TInteger::*;

        // Rule: An `Unspecified` container can hold any input type.
        if self.is_unspecified() {
            return true;
        }

        // Rule: A specific container cannot hold the general `Unspecified` type.
        if input.is_unspecified() {
            return false;
        }

        match (*self, input) {
            (Literal(c), Literal(i)) => c == i,
            (Literal(c), Range(i_from, i_to)) => c == i_from && c == i_to,
            (From(c_from), Literal(i)) => i >= c_from,
            (From(c_from), From(i_from)) => i_from >= c_from,
            (From(c_from), Range(i_from, _)) => i_from >= c_from,
            (To(c_to), Literal(i)) => i <= c_to,
            (To(c_to), To(i_to)) => i_to <= c_to,
            (To(c_to), Range(_, i_to)) => i_to <= c_to,
            (Range(c_from, c_to), Literal(i)) => i >= c_from && i <= c_to,
            (Range(c_from, c_to), Range(i_from, i_to)) => i_from >= c_from && i_to <= c_to,
            _ => false,
        }
    }

    /// Checks if this `TInteger` type is completely covered by a union of other `TInteger` types.
    ///
    /// This function implements a "range reduction" algorithm. It works by iteratively
    /// "subtracting" the container types from the input type (`self`). If the input
    /// range becomes empty, it is considered fully contained.
    pub fn is_contained_by_any(&self, mut int_containers: Vec<TInteger>) -> bool {
        if int_containers.iter().any(TInteger::is_unspecified) {
            return true;
        }

        // If the input is a simple Literal, we just need to find one container that contains it.
        if self.is_literal() {
            return int_containers.iter().any(|c| c.contains(*self));
        }

        // An unspecified input can only be contained by an unspecified container (already checked).
        if self.is_unspecified() {
            return false;
        }

        // 3. Set up the range reduction algorithm.
        // Represent the input range with optional bounds, where None means infinity.
        let mut min_bound = self.get_minimum_value();
        let mut max_bound = self.get_maximum_value();

        // 4. Loop until a full pass over the containers makes no progress.
        loop {
            // Check if the range has been successfully reduced to an empty set.
            if let (Some(min_b), Some(max_b)) = (min_bound, max_bound)
                && min_b > max_b
            {
                return true;
            }

            let mut progress_made = false;

            // We use `retain` to iterate through and remove containers that help reduce the range.
            int_containers.retain(|container| {
                let mut keep_container = true;
                match *container {
                    TInteger::Literal(c_val) => {
                        if Some(c_val) == min_bound {
                            min_bound = Some(c_val.saturating_add(1));
                            progress_made = true;
                            keep_container = false; // Consume the literal.
                        } else if Some(c_val) == max_bound {
                            max_bound = Some(c_val.saturating_sub(1));
                            progress_made = true;
                            keep_container = false;
                        }
                    }
                    TInteger::Range(c_from, c_to) => {
                        if let Some(min_b) = min_bound
                            && min_b >= c_from
                            && min_b <= c_to
                        {
                            min_bound = Some(c_to.saturating_add(1));
                            progress_made = true;
                            keep_container = false;
                        }
                        if let Some(max_b) = max_bound
                            && max_b >= c_from
                            && max_b <= c_to
                        {
                            max_bound = Some(c_from.saturating_sub(1));
                            progress_made = true;
                            keep_container = false;
                        }
                    }
                    TInteger::From(c_from) => {
                        if let Some(min_b) = min_bound
                            && min_b >= c_from
                        {
                            // This container covers the rest of the range.
                            max_bound = Some(c_from.saturating_sub(1));
                            progress_made = true;
                            keep_container = false;
                        }
                    }
                    TInteger::To(c_to) => {
                        if let Some(max_b) = max_bound
                            && max_b <= c_to
                        {
                            // This container covers the start of the range.
                            min_bound = Some(c_to.saturating_add(1));
                            progress_made = true;
                            keep_container = false;
                        }
                    }
                    TInteger::Unspecified => {
                        unreachable!("Unspecified integers should have been filtered out before this point");
                    }
                }

                keep_container
            });

            // If a full pass made no change, we are stuck and cannot fully reduce the range.
            if !progress_made {
                break;
            }
        }

        // If the loop finished but the range is not proven empty, it's not contained.
        false
    }

    /// Computes the set difference of two integer types, returning the parts
    /// of `self` that are not present in `other`.
    ///
    /// This method treats the integer variants as sets of numbers and
    /// calculates the logical difference `self` - `other`.
    ///
    /// The `conservative_subtraction` flag controls the precision of this
    /// operation. When `false`, a precise set difference is always computed.
    ///
    /// When `true`, the operation is bypassed for any non-literal `other` type
    /// (`Unspecified`, `Range`, `From`, `To`), and the method simply returns
    /// a vector containing a copy of `self`. This provides a trade-off
    /// between precision and performance.
    ///
    /// # Parameters
    ///
    /// - `other`: The type to subtract from `self`.
    /// - `conservative_subtraction`: If `true`, the subtraction is bypassed
    ///   for non-literal types, returning `vec![self]` instead.
    pub fn difference(&self, other: Self, conservative_subtraction: bool) -> Vec<Self> {
        let mut res = Vec::with_capacity(2);
        match other {
            TInteger::Unspecified if !conservative_subtraction => {
                // Subtracting the entire set leaves nothing.
            }
            TInteger::Literal(b) => match *self {
                TInteger::Unspecified => {
                    if b > i64::MIN {
                        res.push(TInteger::To(b - 1));
                    }

                    if b < i64::MAX {
                        res.push(TInteger::From(b + 1));
                    }
                }
                TInteger::Literal(a) => {
                    if a != b {
                        res.push(TInteger::Literal(a));
                    }
                }
                TInteger::Range(a1, a2) => {
                    if b < a1 || b > a2 {
                        res.push(TInteger::Range(a1, a2));
                    } else {
                        if b > a1 {
                            res.push(TInteger::Range(a1, b - 1));
                        }

                        if b < a2 {
                            res.push(TInteger::Range(b + 1, a2));
                        }
                    }
                }
                TInteger::From(a) => {
                    if b < a {
                        res.push(TInteger::From(a));
                    } else {
                        if b > a {
                            res.push(TInteger::Range(a, b - 1));
                        }

                        if b < i64::MAX {
                            res.push(TInteger::From(b + 1));
                        }
                    }
                }
                TInteger::To(a) => {
                    if b > a {
                        res.push(TInteger::To(a));
                    } else {
                        if b > i64::MIN {
                            res.push(TInteger::To(b - 1));
                        }

                        if b < a {
                            res.push(TInteger::Range(b + 1, a));
                        }
                    }
                }
            },
            TInteger::Range(b1, b2) if !conservative_subtraction => match *self {
                TInteger::Unspecified => {
                    if b1 > i64::MIN {
                        res.push(TInteger::To(b1 - 1));
                    }
                    if b2 < i64::MAX {
                        res.push(TInteger::From(b2 + 1));
                    }
                }
                TInteger::Literal(a) => {
                    if a >= b1 && a <= b2 {
                        // Fully contained.
                    } else {
                        res.push(TInteger::Literal(a));
                    }
                }
                TInteger::Range(a1, a2) => {
                    if a2 < b1 || a1 > b2 {
                        res.push(TInteger::Range(a1, a2));
                    } else {
                        if a1 < b1 {
                            res.push(TInteger::Range(a1, b1 - 1));
                        }
                        if a2 > b2 {
                            res.push(TInteger::Range(b2 + 1, a2));
                        }
                    }
                }
                TInteger::From(a) => {
                    // [a, MAX] - [b1, b2]
                    if b2 < a {
                        res.push(TInteger::From(a));
                    } else {
                        if a < b1 {
                            res.push(TInteger::Range(a, b1 - 1));
                        }
                        if b2 < i64::MAX {
                            res.push(TInteger::From(b2 + 1));
                        }
                    }
                }
                TInteger::To(a) => {
                    // [MIN, a] - [b1, b2]
                    if b1 > a {
                        res.push(TInteger::To(a));
                    } else {
                        if b1 > i64::MIN {
                            res.push(TInteger::To(b1 - 1));
                        }
                        if a > b2 {
                            res.push(TInteger::Range(b2 + 1, a));
                        }
                    }
                }
            },
            // Subtracting an unbounded range [b, MAX].
            TInteger::From(b) if !conservative_subtraction => match *self {
                TInteger::Unspecified => {
                    if b > i64::MIN {
                        res.push(TInteger::To(b - 1));
                    }
                }
                TInteger::Literal(a) => {
                    if a < b {
                        res.push(TInteger::Literal(a));
                    }
                }
                TInteger::Range(a1, a2) => {
                    if a2 < b {
                        res.push(TInteger::Range(a1, a2));
                    } else if a1 < b {
                        res.push(TInteger::Range(a1, b - 1));
                    }
                }
                TInteger::From(a) => {
                    if a < b {
                        res.push(TInteger::Range(a, b - 1));
                    }
                }
                TInteger::To(a) => {
                    if a < b {
                        res.push(TInteger::To(a));
                    } else if b > i64::MIN {
                        res.push(TInteger::To(b - 1));
                    }
                }
            },
            // Subtracting an unbounded range [MIN, b].
            TInteger::To(b) if !conservative_subtraction => match *self {
                TInteger::Unspecified => {
                    if b < i64::MAX {
                        res.push(TInteger::From(b + 1));
                    }
                }
                TInteger::Literal(a) => {
                    if a > b {
                        res.push(TInteger::Literal(a));
                    }
                }
                TInteger::Range(a1, a2) => {
                    if a1 > b {
                        res.push(TInteger::Range(a1, a2));
                    } else if a2 <= b {
                        // Fully contained.
                    } else {
                        res.push(TInteger::Range(b + 1, a2));
                    }
                }
                TInteger::From(a) => {
                    if a > b {
                        res.push(TInteger::From(a));
                    } else if b < i64::MAX {
                        res.push(TInteger::From(b + 1));
                    }
                }
                TInteger::To(a) => {
                    if a > b {
                        res.push(TInteger::Range(b + 1, a));
                    }
                }
            },
            _ => {
                res.push(*self);
            }
        };

        res
    }

    /// Returns a new `TInteger` that represents the negation of the current type.
    pub fn negated(&self) -> Self {
        match *self {
            TInteger::Literal(v) => TInteger::Literal(-v),
            TInteger::From(f) => TInteger::To(-f),
            TInteger::To(t) => TInteger::From(-t),
            TInteger::Range(f, t) => TInteger::Range(-t, -f),
            TInteger::Unspecified => TInteger::Unspecified,
        }
    }

    /// Narrows the current integer type to be less than the given value (`< n`).
    ///
    /// Returns `Some(new_type)` if the new bound is compatible, or `None` if the
    /// resulting type would represent an impossible range (e.g., `int<10, max>` cannot be `< 5`).
    pub fn to_less_than(&self, n: i64) -> Option<Self> {
        let new_upper_bound = n.saturating_sub(1);
        let narrowed_upper = match self.get_maximum_value() {
            Some(existing_upper) => min(existing_upper, new_upper_bound),
            None => new_upper_bound,
        };

        let new_lower_bound = self.get_minimum_value();
        if let Some(min_b) = new_lower_bound
            && min_b > narrowed_upper
        {
            return None;
        }

        Some(TInteger::from_bounds(new_lower_bound, Some(narrowed_upper)))
    }

    /// Narrows the current integer type to be less than or equal to the given value (`<= n`).
    ///
    /// Returns `Some(new_type)` if the new bound is compatible, or `None` if the
    /// resulting type would represent an impossible range.
    pub fn to_less_than_or_equal(&self, n: i64) -> Option<Self> {
        let narrowed_upper = match self.get_maximum_value() {
            Some(existing_upper) => min(existing_upper, n),
            None => n,
        };

        let new_lower_bound = self.get_minimum_value();
        if let Some(min_b) = new_lower_bound
            && min_b > narrowed_upper
        {
            return None;
        }

        Some(TInteger::from_bounds(new_lower_bound, Some(narrowed_upper)))
    }

    /// Narrows the current integer type to be greater than the given value (`> n`).
    ///
    /// Returns `Some(new_type)` if the new bound is compatible, or `None` if the
    /// resulting type would represent an impossible range.
    pub fn to_greater_than(&self, n: i64) -> Option<Self> {
        let new_lower_bound = n.saturating_add(1);
        let narrowed_lower = match self.get_minimum_value() {
            Some(existing_lower) => max(existing_lower, new_lower_bound),
            None => new_lower_bound,
        };

        let new_upper_bound = self.get_maximum_value();
        if let Some(max_b) = new_upper_bound
            && narrowed_lower > max_b
        {
            return None;
        }

        Some(TInteger::from_bounds(Some(narrowed_lower), new_upper_bound))
    }

    /// Narrows the current integer type to be greater than or equal to the given value (`>= n`).
    ///
    /// Returns `Some(new_type)` if the new bound is compatible, or `None` if the
    /// resulting type would represent an impossible range.
    pub fn to_greater_than_or_equal(&self, n: i64) -> Option<Self> {
        let narrowed_lower = match self.get_minimum_value() {
            Some(existing_lower) => max(existing_lower, n),
            None => n,
        };

        let new_upper_bound = self.get_maximum_value();
        if let Some(max_b) = new_upper_bound
            && narrowed_lower > max_b
        {
            return None;
        }

        Some(TInteger::from_bounds(Some(narrowed_lower), new_upper_bound))
    }

    /// Combines a list of `TInteger` types into the smallest possible set of disjoint types
    /// that collectively cover all inputs.
    ///
    /// # Logic
    ///
    /// This function uses an "interval merging" algorithm. It converts all input types to
    /// `(min, max)` intervals, sorts them, and then merges any that overlap or are adjacent.
    pub fn combine(types: &[TInteger]) -> Vec<TInteger> {
        if types.is_empty() {
            return vec![];
        }

        if types.iter().any(TInteger::is_unspecified) {
            return vec![TInteger::Unspecified];
        }

        if types.iter().all(TInteger::is_literal) {
            return types.to_vec();
        }

        let mut intervals: Vec<(i64, i64)> = types
            .iter()
            .map(|tint| match *tint {
                TInteger::Literal(v) => (v, v),
                TInteger::From(f) => (f, i64::MAX),
                TInteger::To(t) => (i64::MIN, t),
                TInteger::Range(f, t) => (f, t),
                TInteger::Unspecified => (i64::MIN, i64::MAX),
            })
            .collect();

        intervals.sort_unstable_by_key(|k| k.0);

        let mut merged: Vec<(i64, i64)> = Vec::with_capacity(intervals.len());
        merged.push(intervals[0]);

        for current in intervals.iter().skip(1) {
            // SAFETY: we know `merged` is has at least 1 item.
            let last = unsafe { merged.last_mut().unwrap_unchecked() };

            if current.0 <= last.1.saturating_add(1) {
                last.1 = max(last.1, current.1);
            } else {
                merged.push(*current);
            }
        }

        merged
            .into_iter()
            .map(|(min, max)| {
                if min == i64::MIN && max == i64::MAX {
                    TInteger::Unspecified
                } else if min == i64::MIN {
                    TInteger::To(max)
                } else if max == i64::MAX {
                    TInteger::From(min)
                } else if min == max {
                    TInteger::Literal(min)
                } else {
                    TInteger::Range(min, max)
                }
            })
            .collect()
    }
}

impl TType for TInteger {
    fn needs_population(&self) -> bool {
        false
    }

    fn is_expandable(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        match self {
            TInteger::Literal(value) => {
                concat_atom!("int(", i64_atom(*value), ")")
            }
            TInteger::From(value) => {
                if *value == 1 {
                    atom("positive-int")
                } else if *value == 0 {
                    atom("non-negative-int")
                } else {
                    concat_atom!("int<", i64_atom(*value), ", max>")
                }
            }
            TInteger::To(value) => {
                if *value == -1 {
                    atom("negative-int")
                } else if *value == 0 {
                    atom("non-positive-int")
                } else {
                    concat_atom!("int<min, ", i64_atom(*value), ">")
                }
            }
            TInteger::Range(from, to) => concat_atom!("int<", i64_atom(*from), i64_atom(*to), ">"),
            TInteger::Unspecified => atom("int"),
        }
    }
}

impl Add for TInteger {
    type Output = TInteger;

    fn add(self, other: TInteger) -> TInteger {
        use TInteger::*;

        match (self, other) {
            (Unspecified, _) | (_, Unspecified) => Unspecified,
            (Literal(l1), Literal(l2)) => Literal(l1.saturating_add(l2)),
            (Literal(l), From(f)) | (From(f), Literal(l)) => From(l.saturating_add(f)),
            (Literal(l), To(t)) | (To(t), Literal(l)) => To(l.saturating_add(t)),
            (Literal(l), Range(f, t)) | (Range(f, t), Literal(l)) => Range(l.saturating_add(f), l.saturating_add(t)),
            (From(f1), From(f2)) => From(f1.saturating_add(f2)),
            (To(t1), To(t2)) => To(t1.saturating_add(t2)),
            (Range(f1, t1), Range(f2, t2)) => Range(f1.saturating_add(f2), t1.saturating_add(t2)),
            (Range(f1, _), From(f2)) | (From(f2), Range(f1, _)) => From(f1.saturating_add(f2)),
            (Range(_, t1), To(t2)) | (To(t2), Range(_, t1)) => To(t1.saturating_add(t2)),
            _ => Unspecified,
        }
    }
}

impl Sub for TInteger {
    type Output = TInteger;

    fn sub(self, other: TInteger) -> TInteger {
        use TInteger::*;

        match (self, other) {
            (Unspecified, _) | (_, Unspecified) => Unspecified,
            (Literal(l1), Literal(l2)) => Literal(l1.saturating_sub(l2)),
            (From(f), Literal(l)) => From(f.saturating_sub(l)),
            (Literal(l), From(f)) => To(l.saturating_sub(f)),
            (Literal(l), To(t)) => From(l.saturating_sub(t)),
            (From(_) | To(_), From(_) | To(_)) => Unspecified,
            (Range(_, t1), From(f2)) => To(t1.saturating_sub(f2)),
            (Range(f1, _), To(t2)) => From(f1.saturating_sub(t2)),
            (To(t), Literal(l)) => To(t.saturating_sub(l)),
            (Literal(l), Range(f, t)) => Range(l.saturating_sub(t), l.saturating_sub(f)),
            (Range(f, t), Literal(l)) => Range(f.saturating_sub(l), t.saturating_sub(l)),
            (Range(f1, t1), Range(f2, t2)) => Range(f1.saturating_sub(t2), t1.saturating_sub(f2)),
            _ => Unspecified,
        }
    }
}

impl Mul for TInteger {
    type Output = TInteger;

    fn mul(self, other: TInteger) -> TInteger {
        use TInteger::*;

        match (self, other) {
            (Unspecified, _) | (_, Unspecified) => Unspecified,
            (Literal(l1), Literal(l2)) => Literal(l1.saturating_mul(l2)),
            (Literal(0), _) | (_, Literal(0)) => Literal(0),
            (Literal(l), From(f)) | (From(f), Literal(l)) => {
                if l > 0 {
                    From(l.saturating_mul(f))
                } else if l < 0 {
                    To(l.saturating_mul(f))
                } else {
                    Literal(0)
                }
            }
            (Literal(l), To(t)) | (To(t), Literal(l)) => {
                if l > 0 {
                    To(l.saturating_mul(t))
                } else if l < 0 {
                    From(l.saturating_mul(t))
                } else {
                    Literal(0)
                }
            }
            (Literal(l), Range(f, t)) | (Range(f, t), Literal(l)) => {
                if l > 0 {
                    Range(l.saturating_mul(f), l.saturating_mul(t))
                } else if l < 0 {
                    Range(l.saturating_mul(t), l.saturating_mul(f))
                } else {
                    Literal(0)
                }
            }
            (From(f1), From(f2)) => {
                if f1 >= 0 && f2 >= 0 {
                    From(f1.saturating_mul(f2))
                } else {
                    Unspecified
                }
            }
            (Range(f1, t1), Range(f2, t2)) => {
                let p1 = f1.saturating_mul(f2);
                let p2 = f1.saturating_mul(t2);
                let p3 = t1.saturating_mul(f2);
                let p4 = t1.saturating_mul(t2);

                let f = min(min(p1, p2), min(p3, p4));
                let t = max(max(p1, p2), max(p3, p4));

                Range(f, t)
            }
            _ => Unspecified,
        }
    }
}

impl Div for TInteger {
    type Output = TInteger;

    fn div(self, rhs: Self) -> Self::Output {
        use TInteger::*;

        match (self, rhs) {
            (Unspecified, _) | (_, Unspecified) => Unspecified,
            (_, rhs) if rhs.can_be_zero() => Unspecified,
            (Literal(l1), Literal(l2)) => {
                if l1 == 0 {
                    Literal(0)
                } else if l2 == 0 || (l1 == i64::MIN && l2 == -1) {
                    Unspecified
                } else {
                    Literal(l1 / l2)
                }
            }
            (Literal(l), From(f)) => {
                if f > 0 {
                    let v = l / f;

                    if v == 0 {
                        Literal(0)
                    } else if l >= 0 {
                        Range(0, v)
                    } else {
                        Range(v, 0)
                    }
                } else {
                    Unspecified
                }
            }
            (Literal(l), To(t)) => {
                if t < 0 {
                    if l >= 0 { Range(l / t, 0) } else { Range(0, l / t) }
                } else {
                    Unspecified
                }
            }
            (From(f), Literal(l)) => {
                if l > 0 {
                    From(f / l)
                } else if l < 0 {
                    To(f / l)
                } else {
                    Unspecified
                }
            }
            (To(t), Literal(l)) => {
                if l > 0 {
                    To(t / l)
                } else if l < 0 {
                    From(t / l)
                } else {
                    Unspecified
                }
            }
            (Range(f, t), Literal(l)) => {
                if l > 0 {
                    Range(f / l, t / l)
                } else if l < 0 {
                    Range(t / l, f / l)
                } else {
                    Unspecified
                }
            }
            (Literal(l), Range(f, t)) => {
                if f > 0 || t < 0 {
                    let p1 = l / f;
                    let p2 = l / t;

                    let f = min(p1, p2);
                    let t = max(p1, p2);

                    Range(f, t)
                } else {
                    Unspecified
                }
            }
            (Range(f1, t1), Range(f2, t2)) => {
                if f2 > 0 || t2 < 0 {
                    let p1 = f1 / f2;
                    let p2 = f1 / t2;
                    let p3 = t1 / f2;
                    let p4 = t1 / t2;

                    let f = min(min(p1, p2), min(p3, p4));
                    let t = max(max(p1, p2), max(p3, p4));

                    Range(f, t)
                } else {
                    Unspecified
                }
            }
            _ => Unspecified,
        }
    }
}

impl Rem for TInteger {
    type Output = TInteger;

    fn rem(self, rhs: Self) -> Self::Output {
        use TInteger::*;

        match (self, rhs) {
            (Unspecified, other) => match other {
                Unspecified => Unspecified,
                Literal(n) => {
                    if n == 0 {
                        // Division by zero is a potential error.
                        Unspecified
                    } else {
                        // Result is in the range (-|n|, |n|).
                        Range(-(n.abs() - 1), n.abs() - 1)
                    }
                }
                From(_) => {
                    // If n <= 0, the divisor range [n, max] includes 0 (error).
                    // If n > 0, the result is int<-(max-1), max-1> (Unspecified).
                    // Therefore, the outcome is always Unspecified.
                    Unspecified
                }
                To(_) => {
                    // If n >= 0, the divisor range [min, n] includes 0 (error).
                    // If n < 0, the result is int<min+1, -min-1> or int<-max, max> (Unspecified).
                    // Therefore, the outcome is always Unspecified.
                    Unspecified
                }
                Range(n1, n2) => {
                    // If the range contains 0, we represent the potential error as Unspecified.
                    if n1 <= 0 && n2 >= 0 {
                        Unspecified
                    } else if 0 < n1 {
                        // Divisor is positive, result range is bounded by n2.
                        Range(-(n2 - 1), n2 - 1)
                    } else {
                        // n2 < 0
                        // Divisor is negative, result range is bounded by n1.
                        Range(n1 + 1, -n1 - 1)
                    }
                }
            },
            (_, Unspecified) => Unspecified,
            (_, rhs) if rhs.can_be_zero() => Unspecified,
            (Literal(l1), Literal(l2)) => {
                if l1 == i64::MIN && l2 == -1 {
                    Unspecified
                } else {
                    Literal(l1 % l2)
                }
            }
            (_, Literal(1)) | (_, Literal(-1)) => Literal(0),
            (Literal(l), From(f)) => {
                if f > 0 {
                    if l >= 0 { Range(0, (f - 1).min(l)) } else { Range(-(f - 1).min(-l), 0) }
                } else {
                    Unspecified
                }
            }
            (From(f), Literal(l)) => {
                if l > 0 {
                    if f >= 0 { Range(0, l - 1) } else { Range(-(l - 1), l - 1) }
                } else if l < 0 {
                    if f >= 0 { Range(0, -l - 1) } else { Range(l + 1, 0) }
                } else {
                    Unspecified
                }
            }
            (To(_), Literal(l)) => {
                if l > 0 {
                    Range(0, l - 1)
                } else if l < 0 {
                    let f = l.abs();

                    Range(-(f - 1), 0)
                } else {
                    Unspecified
                }
            }
            (Range(f, t), Literal(l)) => {
                if l > 0 {
                    if f >= 0 {
                        Range(0, (l - 1).min(t))
                    } else if t <= 0 {
                        Range((-(l - 1)).max(f), 0)
                    } else {
                        Range(-(l - 1), l - 1)
                    }
                } else if l < 0 {
                    let v = l.abs();

                    if f >= 0 {
                        Range(-(v - 1), 0)
                    } else if t <= 0 {
                        Range(0, v - 1)
                    } else {
                        Range(-(v - 1), v - 1)
                    }
                } else {
                    Unspecified
                }
            }
            _ => Unspecified,
        }
    }
}

impl Neg for TInteger {
    type Output = TInteger;

    fn neg(self) -> Self::Output {
        match self {
            TInteger::Literal(a) => {
                if a == i64::MIN {
                    TInteger::Literal(i64::MAX)
                } else {
                    TInteger::Literal(-a)
                }
            }
            TInteger::From(a) => TInteger::To(-a),
            TInteger::To(a) => TInteger::From(-a),
            TInteger::Range(min, max) => TInteger::Range(-max, -min),
            TInteger::Unspecified => TInteger::Unspecified,
        }
    }
}

impl BitAnd for TInteger {
    type Output = TInteger;

    /// Performs a bitwise AND operation.
    ///
    /// The operation is only computed for two `Literal` values. All other
    /// combinations result in `Unspecified` because the resulting set of
    /// possible values is not guaranteed to be a continuous range.
    fn bitand(self, rhs: Self) -> Self::Output {
        use TInteger::*;
        match (self, rhs) {
            (Literal(l1), Literal(l2)) => Literal(l1 & l2),
            _ => Unspecified,
        }
    }
}

impl BitOr for TInteger {
    type Output = TInteger;

    /// Performs a bitwise OR operation.
    ///
    /// The operation is only computed for two `Literal` values. All other
    /// combinations result in `Unspecified`.
    fn bitor(self, rhs: Self) -> Self::Output {
        use TInteger::*;
        match (self, rhs) {
            (Literal(l1), Literal(l2)) => Literal(l1 | l2),
            _ => Unspecified,
        }
    }
}

impl BitXor for TInteger {
    type Output = TInteger;

    /// Performs a bitwise XOR operation.
    ///
    /// The operation is only computed for two `Literal` values. All other
    /// combinations result in `Unspecified`.
    fn bitxor(self, rhs: Self) -> Self::Output {
        use TInteger::*;
        match (self, rhs) {
            (Literal(l1), Literal(l2)) => Literal(l1 ^ l2),
            _ => Unspecified,
        }
    }
}

impl Shl<TInteger> for TInteger {
    type Output = TInteger;

    /// Performs a bitwise left shift (`<<`) operation.
    ///
    /// This is computed precisely if the shift amount (`rhs`) is a non-negative `Literal`.
    /// Otherwise, the result is `Unspecified`.
    fn shl(self, rhs: TInteger) -> Self::Output {
        use TInteger::*;
        // We can only calculate the result if the shift amount is a known literal.
        if let Literal(shift_amount) = rhs {
            // Shifts must be non-negative and less than the bit width of i64.
            if !(0..64).contains(&shift_amount) {
                return Unspecified;
            }

            let shift_amount = shift_amount as u32;

            return match self {
                Literal(val) => Literal(val.shl(shift_amount)),
                From(from) => From(from.shl(shift_amount)),
                To(to) => To(to.shl(shift_amount)),
                Range(from, to) => {
                    // For left shifts, the order of bounds is always preserved.
                    let r1 = from.shl(shift_amount);
                    let r2 = to.shl(shift_amount);
                    Range(r1, r2)
                }
                Unspecified => Unspecified,
            };
        }

        // If the shift amount is a range, the result is not a continuous interval.
        Unspecified
    }
}

impl Shr for TInteger {
    type Output = TInteger;

    /// Performs a bitwise arithmetic right shift (`>>`) operation.
    ///
    /// This is computed precisely if the shift amount (`rhs`) is a non-negative `Literal`.
    /// Otherwise, the result is `Unspecified`.
    fn shr(self, rhs: TInteger) -> Self::Output {
        use TInteger::*;

        if let Literal(shift_amount) = rhs {
            if !(0..64).contains(&shift_amount) {
                return Unspecified;
            }

            let shift_amount = shift_amount as u32;

            return match self {
                Literal(val) => Literal(val.shr(shift_amount)),
                From(from) => From(from.shr(shift_amount)),
                To(to) => To(to.shr(shift_amount)),
                Range(from, to) => {
                    let r1 = from.shr(shift_amount);
                    let r2 = to.shr(shift_amount);

                    Range(min(r1, r2), max(r1, r2))
                }
                Unspecified => Unspecified,
            };
        }

        Unspecified
    }
}

impl Default for TInteger {
    /// Returns the default value, representing the general `int` type.
    fn default() -> Self {
        Self::unspecified()
    }
}

impl std::fmt::Display for TInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TInteger::Literal(value) => write!(f, "int({value})"),
            TInteger::From(value) => {
                if *value == 1 {
                    write!(f, "positive-int")
                } else if *value == 0 {
                    write!(f, "non-negative-int")
                } else {
                    write!(f, "int<{value}, max>")
                }
            }
            TInteger::To(value) => {
                if *value == -1 {
                    write!(f, "negative-int")
                } else if *value == 0 {
                    write!(f, "non-positive-int")
                } else {
                    write!(f, "int<min, {value}>")
                }
            }
            TInteger::Range(from, to) => write!(f, "int<{from}, {to}>"),
            TInteger::Unspecified => write!(f, "int"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(TInteger::Literal(5) + TInteger::Literal(3), TInteger::Literal(8));
        assert_eq!(TInteger::From(5) + TInteger::From(3), TInteger::From(8));
        assert_eq!(TInteger::To(5) + TInteger::To(3), TInteger::To(8));
        assert_eq!(TInteger::Range(1, 5) + TInteger::Range(2, 3), TInteger::Range(3, 8));
        assert_eq!(TInteger::Literal(5) + TInteger::From(3), TInteger::From(8));
        assert_eq!(TInteger::From(3) + TInteger::Literal(5), TInteger::From(8));
        assert_eq!(TInteger::Literal(5) + TInteger::To(3), TInteger::To(8));
        assert_eq!(TInteger::To(3) + TInteger::Literal(5), TInteger::To(8));
        assert_eq!(TInteger::Literal(5) + TInteger::Range(1, 3), TInteger::Range(6, 8));
        assert_eq!(TInteger::Range(1, 3) + TInteger::Literal(5), TInteger::Range(6, 8));
        assert_eq!(TInteger::Range(1, 5) + TInteger::From(3), TInteger::From(4));
        assert_eq!(TInteger::From(3) + TInteger::Range(1, 5), TInteger::From(4));
        assert_eq!(TInteger::Range(1, 5) + TInteger::To(3), TInteger::To(8));
        assert_eq!(TInteger::To(3) + TInteger::Range(1, 5), TInteger::To(8));
        assert_eq!(TInteger::From(5) + TInteger::To(3), TInteger::Unspecified);
        assert_eq!(TInteger::To(3) + TInteger::From(5), TInteger::Unspecified);
        assert_eq!(TInteger::Unspecified + TInteger::Literal(5), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(5) + TInteger::Unspecified, TInteger::Unspecified);
        assert_eq!(TInteger::Literal(5) + TInteger::Literal(10), TInteger::Literal(15));
        assert_eq!(TInteger::From(10) + TInteger::Literal(5), TInteger::From(15));
        assert_eq!(TInteger::To(10) + TInteger::Literal(5), TInteger::To(15));
        assert_eq!(TInteger::Range(10, 20) + TInteger::Literal(5), TInteger::Range(15, 25));
        assert_eq!(TInteger::From(10) + TInteger::From(20), TInteger::From(30));
        assert_eq!(TInteger::To(10) + TInteger::To(20), TInteger::To(30));
        assert_eq!(TInteger::Range(1, 5) + TInteger::Range(10, 20), TInteger::Range(11, 25));
        assert_eq!(TInteger::From(10) + TInteger::To(20), TInteger::Unspecified);
    }

    #[test]
    fn test_sub() {
        assert_eq!(TInteger::Literal(5) - TInteger::Literal(3), TInteger::Literal(2));
        assert_eq!(TInteger::From(5) - TInteger::From(3), TInteger::Unspecified);
        assert_eq!(TInteger::To(5) - TInteger::To(3), TInteger::Unspecified);
        assert_eq!(TInteger::Range(5, 10) - TInteger::Range(2, 3), TInteger::Range(2, 8));
        assert_eq!(TInteger::Literal(10) - TInteger::From(3), TInteger::To(7));
        assert_eq!(TInteger::From(5) - TInteger::Literal(3), TInteger::From(2));
        assert_eq!(TInteger::Literal(10) - TInteger::To(5), TInteger::From(5));
        assert_eq!(TInteger::To(10) - TInteger::Literal(3), TInteger::To(7));
        assert_eq!(TInteger::Literal(10) - TInteger::Range(2, 5), TInteger::Range(5, 8));
        assert_eq!(TInteger::Range(5, 10) - TInteger::Literal(3), TInteger::Range(2, 7));
        assert_eq!(TInteger::Literal(20) - TInteger::Literal(5), TInteger::Literal(15));
        assert_eq!(TInteger::Literal(20) - TInteger::From(5), TInteger::To(15));
        assert_eq!(TInteger::Literal(20) - TInteger::To(5), TInteger::From(15));
        assert_eq!(TInteger::Literal(20) - TInteger::Range(5, 10), TInteger::Range(10, 15));
        assert_eq!(TInteger::From(20) - TInteger::Literal(5), TInteger::From(15));
        assert_eq!(TInteger::To(20) - TInteger::Literal(5), TInteger::To(15));
        assert_eq!(TInteger::Range(20, 30) - TInteger::Literal(5), TInteger::Range(15, 25));
        assert_eq!(TInteger::Range(20, 30) - TInteger::Range(5, 10), TInteger::Range(10, 25));
        assert_eq!(TInteger::From(10) - TInteger::From(20), TInteger::Unspecified);
        assert_eq!(TInteger::To(10) - TInteger::To(20), TInteger::Unspecified);
        assert_eq!(TInteger::From(10) - TInteger::To(20), TInteger::Unspecified);
    }

    #[test]
    fn test_mul() {
        assert_eq!(TInteger::Literal(5) * TInteger::Literal(3), TInteger::Literal(15));
        assert_eq!(TInteger::From(2) * TInteger::From(3), TInteger::From(6));
        assert_eq!(TInteger::From(-2) * TInteger::From(3), TInteger::Unspecified);
        assert_eq!(TInteger::Range(2, 3) * TInteger::Range(4, 5), TInteger::Range(8, 15));
        assert_eq!(TInteger::Literal(0) * TInteger::From(5), TInteger::Literal(0));
        assert_eq!(TInteger::Literal(2) * TInteger::From(3), TInteger::From(6));
        assert_eq!(TInteger::Literal(2) * TInteger::Range(3, 5), TInteger::Range(6, 10));
        assert_eq!(TInteger::Literal(-2) * TInteger::Range(3, 5), TInteger::Range(-10, -6));
        assert_eq!(TInteger::Literal(4) * TInteger::Literal(5), TInteger::Literal(20));
        assert_eq!(TInteger::Range(2, 5) * TInteger::Literal(10), TInteger::Range(20, 50));
        assert_eq!(TInteger::Range(-2, 5) * TInteger::Literal(10), TInteger::Range(-20, 50));
        assert_eq!(TInteger::Range(-5, -2) * TInteger::Literal(-10), TInteger::Range(20, 50));
        assert_eq!(TInteger::Range(2, 3) * TInteger::Range(4, 5), TInteger::Range(8, 15));
        assert_eq!(TInteger::Range(-2, 3) * TInteger::Range(-4, 5), TInteger::Range(-12, 15));
        assert_eq!(TInteger::From(10) * TInteger::Literal(2), TInteger::From(20));
        assert_eq!(TInteger::From(10) * TInteger::Literal(-2), TInteger::To(-20));
        assert_eq!(TInteger::To(10) * TInteger::Literal(2), TInteger::To(20));
        assert_eq!(TInteger::To(10) * TInteger::Literal(-2), TInteger::From(-20));
    }

    #[test]
    fn test_div() {
        assert_eq!(TInteger::Literal(15) / TInteger::Literal(3), TInteger::Literal(5));
        assert_eq!(TInteger::Literal(15) / TInteger::Literal(0), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(0) / TInteger::Literal(5), TInteger::Literal(0));
        assert_eq!(TInteger::Literal(0) / TInteger::From(5), TInteger::Literal(0));
        assert_eq!(TInteger::From(10) / TInteger::Literal(2), TInteger::From(5));
        assert_eq!(TInteger::From(10) / TInteger::Literal(-2), TInteger::To(-5));
        assert_eq!(TInteger::To(10) / TInteger::Literal(2), TInteger::To(5));
        assert_eq!(TInteger::To(10) / TInteger::Literal(-2), TInteger::From(-5));
        assert_eq!(TInteger::Range(10, 20) / TInteger::Literal(2), TInteger::Range(5, 10));
        assert_eq!(TInteger::Range(10, 20) / TInteger::Literal(-2), TInteger::Range(-10, -5));
        assert_eq!(TInteger::Literal(10) / TInteger::From(2), TInteger::Range(0, 5));
        assert_eq!(TInteger::Literal(-10) / TInteger::From(2), TInteger::Range(-5, 0));
        assert_eq!(TInteger::Literal(20) / TInteger::Literal(4), TInteger::Literal(5));
        assert_eq!(TInteger::Range(10, 20) / TInteger::Literal(2), TInteger::Range(5, 10));
        assert_eq!(TInteger::Range(10, 20) / TInteger::Literal(-2), TInteger::Range(-10, -5));
        assert_eq!(TInteger::Literal(100) / TInteger::From(10), TInteger::Range(0, 10));
        assert_eq!(TInteger::Literal(-100) / TInteger::From(10), TInteger::Range(-10, 0));
        assert_eq!(TInteger::Literal(100) / TInteger::To(-10), TInteger::Range(-10, 0));
        assert_eq!(TInteger::Literal(-100) / TInteger::To(-10), TInteger::Range(0, 10));
        assert_eq!(TInteger::Range(100, 200) / TInteger::Range(2, 4), TInteger::Range(25, 100));
        assert_eq!(TInteger::Literal(10) / TInteger::Literal(0), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(10) / TInteger::Range(-1, 1), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(10) / TInteger::From(-5), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(10) / TInteger::To(5), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(i64::MIN) / TInteger::Literal(-1), TInteger::Unspecified);
        assert_eq!(TInteger::From(10) / TInteger::From(2), TInteger::Unspecified);
    }

    #[test]
    fn test_rem() {
        assert_eq!(TInteger::Literal(15) % TInteger::Literal(7), TInteger::Literal(1));
        assert_eq!(TInteger::Literal(15) % TInteger::Literal(0), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(0) % TInteger::Literal(5), TInteger::Literal(0));
        assert_eq!(TInteger::From(10) % TInteger::Literal(3), TInteger::Range(0, 2));
        assert_eq!(TInteger::To(10) % TInteger::Literal(3), TInteger::Range(0, 2));
        assert_eq!(TInteger::Range(5, 15) % TInteger::Literal(4), TInteger::Range(0, 3));
        assert_eq!(TInteger::From(10) % TInteger::Literal(-3), TInteger::Range(0, 2));
        assert_eq!(TInteger::Literal(10) % TInteger::From(3), TInteger::Range(0, 2));
        assert_eq!(TInteger::Literal(-5) % TInteger::From(3), TInteger::Range(-2, 0));
        assert_eq!(TInteger::Literal(22) % TInteger::Literal(5), TInteger::Literal(2));
        assert_eq!(TInteger::Literal(22) % TInteger::Literal(-5), TInteger::Literal(2));
        assert_eq!(TInteger::Literal(-22) % TInteger::Literal(5), TInteger::Literal(-2));
        assert_eq!(TInteger::Range(0, 100) % TInteger::Literal(10), TInteger::Range(0, 9));
        assert_eq!(TInteger::Range(-100, -1) % TInteger::Literal(10), TInteger::Range(-9, 0));
        assert_eq!(TInteger::Range(-10, 10) % TInteger::Literal(5), TInteger::Range(-4, 4));
        assert_eq!(TInteger::From(0) % TInteger::Literal(7), TInteger::Range(0, 6));
        assert_eq!(TInteger::From(-100) % TInteger::Literal(7), TInteger::Range(-6, 6));
        assert_eq!(TInteger::Literal(123) % TInteger::Literal(1), TInteger::Literal(0));
        assert_eq!(TInteger::Range(-10, 10) % TInteger::Literal(-1), TInteger::Literal(0));
        assert_eq!(TInteger::Literal(10) % TInteger::Literal(0), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(10) % TInteger::Range(-1, 1), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(100) % TInteger::Range(2, 5), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(i64::MIN) % TInteger::Literal(-1), TInteger::Unspecified);
        assert_eq!(TInteger::Unspecified % TInteger::Literal(5), TInteger::Range(-4, 4));
        assert_eq!(TInteger::Unspecified % TInteger::Range(1, 10), TInteger::Range(-9, 9));
        assert_eq!(TInteger::Unspecified % TInteger::From(5), TInteger::Unspecified);
        assert_eq!(TInteger::Unspecified % TInteger::To(5), TInteger::Unspecified);
    }

    #[test]
    fn test_neg() {
        assert_eq!(-TInteger::Literal(5), TInteger::Literal(-5));
        assert_eq!(-TInteger::Literal(-3), TInteger::Literal(3));
        assert_eq!(-TInteger::From(5), TInteger::To(-5));
        assert_eq!(-TInteger::To(5), TInteger::From(-5));
        assert_eq!(-TInteger::Range(2, 7), TInteger::Range(-7, -2));
        assert_eq!(-TInteger::Range(-5, -2), TInteger::Range(2, 5));
        assert_eq!(-TInteger::Unspecified, TInteger::Unspecified);
        assert_eq!(-TInteger::Literal(10), TInteger::Literal(-10));
        assert_eq!(-TInteger::From(10), TInteger::To(-10));
        assert_eq!(-TInteger::To(-5), TInteger::From(5));
        assert_eq!(-TInteger::Range(-10, 20), TInteger::Range(-20, 10));
        assert_eq!(-TInteger::Unspecified, TInteger::Unspecified);
        assert_eq!(-TInteger::Literal(i64::MIN), TInteger::Literal(i64::MAX));
    }

    #[test]
    fn test_bitwise_ops() {
        assert_eq!(TInteger::Literal(6) & TInteger::Literal(3), TInteger::Literal(2));
        assert_eq!(TInteger::Literal(5) & TInteger::Range(0, 10), TInteger::Unspecified);
        assert_eq!(TInteger::Range(0, 10) & TInteger::Literal(5), TInteger::Unspecified);
        assert_eq!(TInteger::From(1) & TInteger::Literal(5), TInteger::Unspecified);
        assert_eq!(TInteger::To(10) & TInteger::Literal(5), TInteger::Unspecified);
        assert_eq!(TInteger::Range(0, 10) & TInteger::Range(0, 10), TInteger::Unspecified);
        assert_eq!(TInteger::Unspecified & TInteger::Literal(5), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(5) | TInteger::Literal(2), TInteger::Literal(7));
        assert_eq!(TInteger::Literal(5) | TInteger::Range(0, 10), TInteger::Unspecified);
        assert_eq!(TInteger::Range(0, 10) | TInteger::Literal(5), TInteger::Unspecified);
        assert_eq!(TInteger::Unspecified | TInteger::Literal(5), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(6) ^ TInteger::Literal(3), TInteger::Literal(5));
        assert_eq!(TInteger::Literal(5) ^ TInteger::Range(0, 10), TInteger::Unspecified);
        assert_eq!(TInteger::Range(0, 10) ^ TInteger::Literal(5), TInteger::Unspecified);
        assert_eq!(TInteger::Unspecified ^ TInteger::Literal(5), TInteger::Unspecified);
    }

    #[test]
    fn test_shift_ops() {
        assert_eq!(TInteger::Literal(5) << TInteger::Literal(2), TInteger::Literal(20));
        assert_eq!(TInteger::Range(10, 20) << TInteger::Literal(1), TInteger::Range(20, 40));
        assert_eq!(TInteger::From(100) << TInteger::Literal(2), TInteger::From(400));
        assert_eq!(TInteger::To(-10) << TInteger::Literal(1), TInteger::To(-20));
        assert_eq!(TInteger::Literal(5) << TInteger::Range(1, 3), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(5) << TInteger::Literal(-1), TInteger::Unspecified);

        assert_eq!(TInteger::Literal(20) >> TInteger::Literal(2), TInteger::Literal(5));
        assert_eq!(TInteger::Literal(-20) >> TInteger::Literal(2), TInteger::Literal(-5));
        assert_eq!(TInteger::Range(10, 20) >> TInteger::Literal(1), TInteger::Range(5, 10));
        assert_eq!(TInteger::From(400) >> TInteger::Literal(2), TInteger::From(100));
        assert_eq!(TInteger::To(-20) >> TInteger::Literal(1), TInteger::To(-10));
        assert_eq!(TInteger::Range(-10, 10) >> TInteger::Literal(1), TInteger::Range(-5, 5));
        assert_eq!(TInteger::Literal(20) >> TInteger::Range(1, 2), TInteger::Unspecified);
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(TInteger::Literal(0) + TInteger::From(5), TInteger::From(5));
        assert_eq!(TInteger::Literal(0) * TInteger::Range(1, 10), TInteger::Literal(0));
        assert_eq!(TInteger::Range(-5, -1) + TInteger::Range(2, 4), TInteger::Range(-3, 3));
        assert_eq!(TInteger::Range(-3, 2) * TInteger::Range(-1, 4), TInteger::Range(-12, 8));
        assert_eq!(TInteger::From(5) * TInteger::To(3), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(10) / TInteger::Range(-2, 2), TInteger::Unspecified);
        assert_eq!(TInteger::Literal(10) / TInteger::Range(2, 5), TInteger::Range(2, 5));
    }

    #[test]
    fn test_arithmetic_properties() {
        let a = TInteger::Literal(5);
        let b = TInteger::Range(2, 4);

        let _ = a + b;
        let _ = a - b;
        let _ = a * b;
        let _ = a / b;
        let _ = a % b;
        let _ = -a;

        assert_eq!(a + b, b + a);
        assert_eq!(a * b, b * a);

        assert_eq!(a - b + b - a + a + b, TInteger::Range(5, 11));
    }

    #[test]
    fn test_contains() {
        use TInteger::*;

        assert!(Literal(5).contains(Literal(5)));
        assert!(!Literal(5).contains(From(5)));
        assert!(!Literal(5).contains(To(5)));
        assert!(Literal(5).contains(Range(5, 5)));
        assert!(!Literal(5).contains(Literal(6)));
    }

    #[test]
    fn test_display() {
        assert_eq!(TInteger::Literal(5).to_string(), "int(5)");

        assert_eq!(TInteger::From(2).to_string(), "int<2, max>");
        assert_eq!(TInteger::From(1).to_string(), "positive-int");
        assert_eq!(TInteger::From(0).to_string(), "non-negative-int");
        assert_eq!(TInteger::From(-1).to_string(), "int<-1, max>");

        assert_eq!(TInteger::To(2).to_string(), "int<min, 2>");
        assert_eq!(TInteger::To(0).to_string(), "non-positive-int");
        assert_eq!(TInteger::To(-1).to_string(), "negative-int");
        assert_eq!(TInteger::To(-2).to_string(), "int<min, -2>");

        assert_eq!(TInteger::Range(1, 5).to_string(), "int<1, 5>");
        assert_eq!(TInteger::Range(5, 10).to_string(), "int<5, 10>");

        assert_eq!(TInteger::Unspecified.to_string(), "int");
    }
}
