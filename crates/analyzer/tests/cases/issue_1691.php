<?php

declare(strict_types=1);

/**
 * @psalm-type Issue1691Code = int<0, max>
 *
 * @template T of Issue1691Code
 */
class Issue1691A
{
    /** @param T $a */
    function __construct(
        public int $a,
    ) {}

    /**
     * @return T
     */
    public function getVal(): int
    {
        return $this->a;
    }
}

/**
 * @param Issue1691A<5> $a
 * @return 5
 */
function issue_1691_x(Issue1691A $a): int
{
    return $a->getVal();
}
