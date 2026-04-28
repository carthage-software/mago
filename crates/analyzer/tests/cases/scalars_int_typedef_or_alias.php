<?php

declare(strict_types=1);

/**
 * @phpstan-type Int8 int<-128, 127>
 */
final class Box {
    /** @var Int8 */
    public int $byte = 0;
}

$b = new Box();
echo $b->byte;
