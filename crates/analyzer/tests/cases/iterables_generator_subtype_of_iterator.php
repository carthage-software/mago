<?php

declare(strict_types=1);

/**
 * @param Iterator<int, string> $iter
 */
function consume(Iterator $iter): void
{
    foreach ($iter as $v) {
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
