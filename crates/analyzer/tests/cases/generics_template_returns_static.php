<?php

declare(strict_types=1);

/**
 * @template T
 */
class GenStaticRet
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /**
     * @param T $value
     *
     * @return self<T>
     */
    public function withVal(mixed $value): self
    {
        $copy = clone $this;
        $copy->value = $value;
        return $copy;
    }
}

$g = new GenStaticRet(1);
$g2 = $g->withVal(2);
echo $g2->value + 1;
