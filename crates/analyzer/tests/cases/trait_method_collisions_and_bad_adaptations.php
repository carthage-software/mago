<?php

declare(strict_types=1);

trait A
{
    public function items(): int
    {
        return 1;
    }
}

trait B
{
    public function items(): string
    {
        return 'b';
    }
}

trait C
{
    abstract public function items(): int;
}

// @mago-expect analysis:invalid-trait-use
final class Conflict
{
    use A, B;
}

final class Resolved
{
    use A, B {
        B::items insteadof A;
        A::items as itemsFromA;
    }
}

final class Overrides
{
    use A, B;

    public function items(): int
    {
        return 3;
    }
}

final class WithAbstract
{
    use A, C;
}

final class BadAlias
{
    use A {
        // @mago-expect analysis:invalid-trait-use
        A::missing as other;
    }
}
