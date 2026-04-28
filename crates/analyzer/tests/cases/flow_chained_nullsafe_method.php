<?php

declare(strict_types=1);

final class A
{
    public function getB(): null|B
    {
        return null;
    }
}

final class B
{
    public function getC(): null|C
    {
        return null;
    }
}

final class C
{
    public function name(): string
    {
        return 'c';
    }
}

function flow_chained_nullsafe_method(null|A $a): null|string
{
    return $a?->getB()?->getC()?->name();
}
