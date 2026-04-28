<?php

declare(strict_types=1);

/**
 * @template T
 */
interface ContainerCJ
{
    /** @return T */
    public function get(): mixed;
}

/**
 * @implements ContainerCJ<int>
 */
final class IntBoxCJ implements ContainerCJ
{
    public function __construct(private int $v) {}

    public function get(): int
    {
        return $this->v;
    }
}

$b = new IntBoxCJ(42);
echo $b->get() + 1;
