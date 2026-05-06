<?php

declare(strict_types=1);

final class A {}

final class B {}

function probe(): bool
{
    return A::class === B::class;
}
