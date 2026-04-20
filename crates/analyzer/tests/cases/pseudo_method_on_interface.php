<?php

declare(strict_types=1);

/**
 * @method string describe(int $x)
 */
interface DescribableStub {}

/**
 * @return string
 */
function take(DescribableStub $s): string
{
    return $s->describe(1);
}
