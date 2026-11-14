<?php

declare(strict_types=1);

function byValue(int $n): int
{
    switch (true) {
        case $n === 1:
            return 1;
        // @mago-expect analysis:never-matching-switch-case,redundant-comparison
        case $n === 1:
            return 2;
    }

    return 3;
}

class Foo {}

final class Baz extends Foo {}

function byType(Foo|string $u): int
{
    switch (true) {
        case $u instanceof Foo:
            return 1;
        // @mago-expect analysis:never-matching-switch-case
        case $u instanceof Baz:
            return 2;
        default:
            return 3;
    }
}
