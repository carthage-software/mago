<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenOpt
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

/** @var GenOpt<int> $o */
$o = new GenOpt();
$v = $o->get();
if (null !== $v) {
    echo $v + 1;
}
