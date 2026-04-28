<?php

declare(strict_types=1);

/**
 * @return Generator<int, string>
 */
function gen(): Generator
{
    yield 'a';
}

function consume(): void
{
    $g = gen();
    $g->current();
    try {
        $g->throw(new RuntimeException('boom'));
    } catch (RuntimeException $_e) {
    }
}
