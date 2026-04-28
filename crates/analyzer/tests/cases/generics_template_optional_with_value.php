<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenOpt2
{
    /** @param T|null $value */
    public function __construct(public mixed $value = null)
    {
    }

    /** @return T|null */
    public function get(): mixed
    {
        return $this->value;
    }
}

$o = new GenOpt2(5);
$v = $o->get();
if (null !== $v) {
    echo $v + 1;
}
