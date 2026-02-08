<?php declare(strict_types=1);

/** @template T */
interface I
{
    /** @return self<T> */
    function f(): self;
}

/** @implements I<mixed> */
enum E implements I
{
    case A;

    #[Override]
    function f(): self
    {
        return $this;
    }
}
