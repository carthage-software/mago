<?php

declare(strict_types=1);

function takeString(string $s): void
{
    echo $s;
}

/** @return array{foo?: string, bar?: string} */
function getShape(): array
{
    return [];
}

$shape = getShape();

if (isset($shape['foo'])) {
    takeString($shape['foo']); // Ok
}

if (isset($shape['bar'])) {
    takeString($shape['bar']); // Ok
}

if (isset($shape['foo']) && isset($shape['bar'])) {
    takeString($shape['foo']); // Ok
    takeString($shape['bar']); // Ok
}

if (isset($shape['foo'], $shape['bar'])) {
    takeString($shape['foo']); // Ok
    takeString($shape['bar']); // Ok
}
