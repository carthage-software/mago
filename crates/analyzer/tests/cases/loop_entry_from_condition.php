<?php

declare(strict_types=1);

function loop_entry_pre_increment(): string
{
    $try = 0;
    while (++$try < 10) {
        $name = 'thing';
    }

    return $name;
}

function loop_entry_pre_decrement(): string
{
    for ($i = 5; --$i > 3;) {
        $name = 'thing';
    }

    return $name;
}

final class LoopEntryNode
{
    public function __construct(
        public string $name,
        public null|LoopEntryNode $next = null,
    ) {}
}

function loop_entry_not_null(null|LoopEntryNode $node): string
{
    while ($node !== null) {
        $name = $node->name;
        $node = $node->next;
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $name;
}

function loop_entry_float(float $x): string
{
    while ($x < 3.0) {
        $name = 'thing';
        $x += 1.0;
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $name;
}

function loop_entry_condition_assignment(string $base): string
{
    while (false === @mkdir($dir = $base . rand(10000, 99999), 0777, true)) {
    }

    return $dir;
}
