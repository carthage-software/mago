<?php

declare(strict_types=1);

final class A
{
    public int $a = 1;
}

final class B
{
    public int $b = 2;
}

function flow_negated_instanceof(A|B $o): int
{
    if (!($o instanceof A)) {
        return $o->b;
    }

    return $o->a;
}
