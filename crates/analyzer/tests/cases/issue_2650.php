<?php

declare(strict_types=1);

namespace Issue2650;

interface Description
{
    public static function name(): string;
}

final class DateDiff implements Description
{
    #[\Override]
    public static function name(): string
    {
        return 'date_diff';
    }
}

function print_names(): void
{
    $classes = [Description::class, DateDiff::class];

    foreach ($classes as $class) {
        if (!\is_subclass_of($class, Description::class)) {
            continue;
        }

        echo $class::name();
    }
}

/**
 * @param class-string<DateDiff> $class
 */
function accepts_date_diff(string $class): void
{
    echo $class;
}

function narrows_to_subclass(bool $parent): void
{
    $class = $parent ? Description::class : DateDiff::class;

    if (\is_subclass_of($class, Description::class)) {
        accepts_date_diff($class);
    }
}

function keeps_parent_when_false(bool $parent): string
{
    $class = $parent ? Description::class : DateDiff::class;

    if (\is_subclass_of($class, Description::class)) {
        return 'subclass';
    }

    return $class === Description::class ? 'parent' : 'other';
}
