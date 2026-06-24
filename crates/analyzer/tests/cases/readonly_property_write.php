<?php

declare(strict_types=1);

final class Point
{
    public function __construct(public readonly int $x = 0) {}

    public function move(int $x): void
    {
        /** @mago-expect analysis:invalid-property-write */
        $this->x = $x;
    }
}

readonly class Frozen
{
    public function __construct(public int $value = 0) {}
}

$point = new Point();
/** @mago-expect analysis:invalid-property-write */
$point->x = 10;

$frozen = new Frozen();
/** @mago-expect analysis:invalid-property-write */
$frozen->value = 20;
