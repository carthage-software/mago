<?php

declare(strict_types=1);

class WidgetCF
{
    public int $id = 5;
}

/**
 * @template T of object
 *
 * @param class-string<T> $cls
 *
 * @return class-string<T>
 */
function passClassCF(string $cls): string
{
    return $cls;
}

echo passClassCF(WidgetCF::class);
