<?php

function example(): void {
    $comparator = match ($this->operator) {
        BinaryOperatorKind::LessThan =>
            /**
             * @throws UnsupportedOperationException If a value comparison is not supported.
             */
            static fn(Value $a, Value $b): bool => $a->isLessThan($b),
        BinaryOperatorKind::LessThanOrEqual =>
            /**
             * @throws UnsupportedOperationException If a value comparison is not supported.
             */
            static fn(Value $a, Value $b): bool => $a->isLessThan($b) || $a->isEqual($b),
        BinaryOperatorKind::GreaterThan =>
            /**
             * @throws UnsupportedOperationException If a value comparison is not supported.
             */
            static fn(Value $a, Value $b): bool => $a->isGreaterThan($b),
        BinaryOperatorKind::GreaterThanOrEqual =>
            /**
             * @throws UnsupportedOperationException If a value comparison is not supported.
             */
            static fn(Value $a, Value $b): bool => $a->isGreaterThan($b) || $a->isEqual($b),
        default => throw InternalException::forInvalidOperator($this->operator->getSymbol()),
    };
}
