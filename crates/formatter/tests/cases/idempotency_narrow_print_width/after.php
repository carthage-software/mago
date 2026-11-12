<?php

final class Factory
{
    private function register(): void
    {
        // `new Foo` and `new Foo()` are the same argument.
        $this->registerDefaultComparator(
            new ClosureComparator(),
        );
        $this->registerDefaultComparator(
            new ClosureComparator(),
        );
    }

    public function add($enumerator, $value): void
    {
        // The dropped parentheses must not change the layout.
        $ord =
            $this->enumeration::get(
                $enumerator,
            )->getOrdinal();
        $ord =
            $this->enumeration::get(
                $enumerator,
            )->getOrdinal();
    }

    public function sigils(): array
    {
        // A comment after the separator stays there when the value is wrapped.
        return [
            '<::>' => static fn() => (
                $valueRight < $valueLeft
            ), // a<5 && a>7; 7<a<5; 7<5;
            '<::>=' => static fn() => (
                $valueRight < $valueLeft
            ),
        ];
    }

    public function sum(): int
    {
        return (
            $this->firstOperand
            + $this->secondOperand
            + $this->third
        ); // the total
    }
}
