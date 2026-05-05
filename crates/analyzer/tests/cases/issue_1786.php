<?php

declare(strict_types=1);

interface A
{
    public int $property { get; }
}

function test(A $a): void
{
    if (isset($a->property)) {
        echo $a->property;
    }

    if ($a->property ?? false) {
        echo $a->property;
    }
}
