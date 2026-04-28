<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenLocalUse
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /**
     * @return T
     */
    public function self_get(): mixed
    {
        /** @var T $local */
        $local = $this->value;
        return $local;
    }
}

$g = new GenLocalUse(7);
echo $g->self_get() + 1;
