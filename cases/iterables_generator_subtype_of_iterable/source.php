<?php

declare(strict_types=1);

/**
 * @param iterable<int, string> $it
 */
function consume(iterable $it): void
{
    foreach ($it as $v) {
        echo $v;
    }
}

/**
 * @param Generator<int, string> $g
 */
function pass_generator(Generator $g): void
{
    consume($g);
}
