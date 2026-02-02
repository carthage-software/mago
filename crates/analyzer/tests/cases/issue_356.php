<?php

declare(strict_types=1);

enum Foo
{
    case A;
    case B;
    case C;
}

class Example
{
    /**
     * @mago-expect analysis:redundant-condition - the last elseif
     * @mago-expect analysis:redundant-comparison - `Foo::C === $foo`
     * @mago-expect analysis:unreachable-else-clause - the else block
     *
     * @mago-expect lint:no-else-clause - shh
     */
    public function process(Foo $foo): void
    {
        if (Foo::A === $foo) {
            // Handle a case
        } elseif (Foo::B === $foo) {
            // Handle b case
        } elseif (Foo::C === $foo) {
            // Handle c case
        } else {
            throw new RuntimeException('Unknown case');
        }
    }

    /**
     * @mago-expect analysis:redundant-condition - the last if
     * @mago-expect analysis:redundant-comparison - `Foo::C === $foo`
     * @mago-expect analysis:unreachable-else-clause - the else block
     *
     * @mago-expect lint:no-else-clause - shh
     */
    public function processWithNestedIf(Foo $foo): void
    {
        if (Foo::A === $foo) {
            // Handle a case
        } else {
            if (Foo::B === $foo) {
                // Handle b case
            } else {
                if (Foo::C === $foo) {
                    // Handle c case
                } else {
                    throw new RuntimeException('Unknown case');
                }
            }
        }
    }

    /**
     * Two-case enum should also work.
     *
     * @mago-expect analysis:redundant-condition - the elseif
     * @mago-expect analysis:redundant-comparison - `TwoCase::B === $val`
     * @mago-expect analysis:unreachable-else-clause - the else block
     *
     * @mago-expect lint:no-else-clause - shh
     */
    public function processTwoCaseEnum(TwoCase $val): void
    {
        if (TwoCase::A === $val) {
            // Handle A
        } elseif (TwoCase::B === $val) {
            // Handle B
        } else {
            throw new RuntimeException('Unreachable');
        }
    }

    /**
     * No else clause - should not trigger the unreachable-else-clause error.
     *
     * @mago-expect analysis:redundant-condition - the last elseif
     * @mago-expect analysis:redundant-comparison - `Foo::C === $foo`
     */
    public function processWithoutElse(Foo $foo): void
    {
        if (Foo::A === $foo) {
            // Handle a case
        } elseif (Foo::B === $foo) {
            // Handle b case
        } elseif (Foo::C === $foo) {
            // Handle c case
        }
    }

    /**
     * Not all cases handled - should NOT trigger unreachable-else-clause.
     */
    public function processPartial(Foo $foo): void
    {
        if (Foo::A === $foo) {
            // Handle a case
        } elseif (Foo::B === $foo) {
            // Handle b case
        } else {
            // This handles Foo::C - not unreachable
            echo "C or unknown";
        }
    }
}

enum TwoCase
{
    case A;
    case B;
}
