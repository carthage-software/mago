<?php

declare(strict_types=1);

namespace App;

class Issue2231ClassA
{
    public int $a = 1;
}

class Issue2231ClassB
{
    public int $b = 2;
}

function issue_2231_example(Issue2231ClassA|Issue2231ClassB $object): int
{
    return match ($object instanceof Issue2231ClassA) {
        true => $object->a,
        false => $object->b,
    };
}

function issue_2231_reversed(Issue2231ClassA|Issue2231ClassB $object): int
{
    return match ($object instanceof Issue2231ClassA) {
        false => $object->b,
        true => $object->a,
    };
}

function issue_2231_true_default(Issue2231ClassA|Issue2231ClassB $object): int
{
    return match ($object instanceof Issue2231ClassA) {
        true => $object->a,
        default => $object->b,
    };
}

function issue_2231_false_default(Issue2231ClassA|Issue2231ClassB $object): int
{
    return match ($object instanceof Issue2231ClassA) {
        false => $object->b,
        default => $object->a,
    };
}
