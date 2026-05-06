<?php

declare(strict_types=1);

interface NamedBM
{
    public function name(): string;
}

interface AgedBM
{
    public function age(): int;
}

/**
 * @param NamedBM&AgedBM $x
 */
function describeBM(object $x): string
{
    return $x->name() . ':' . $x->age();
}

final class PersonBM implements NamedBM, AgedBM
{
    public function name(): string
    {
        return 'alice';
    }

    public function age(): int
    {
        return 30;
    }
}

echo describeBM(new PersonBM());
