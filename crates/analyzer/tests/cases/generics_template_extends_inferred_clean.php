<?php

declare(strict_types=1);

/**
 * @template T
 */
abstract class GenInfBase
{
    /** @return T */
    abstract public function get(): mixed;
}

/**
 * @extends GenInfBase<int>
 */
final class GenInfImpl extends GenInfBase
{
    public function get(): int
    {
        return 1;
    }
}

/**
 * @param GenInfBase<int> $b
 */
function take_inf_int(GenInfBase $b): int
{
    return $b->get();
}

take_inf_int(new GenInfImpl());
