<?php

declare(strict_types=1);

/** @inheritors B|C|D */
interface A {}

interface B extends A {}

final class C implements B {}

final class D implements A {}

function only_d(A $a): ?D
{
    if ($a instanceof B) {
        return null;
    }

    return $a;
}
