<?php

declare(strict_types=1);

final class SwitchOnStringableName
{
    public function __toString(): string
    {
        return 'locator';
    }
}

final class SwitchOnPlainObject
{
}

function switch_on_stringable_matches_string_cases(SwitchOnStringableName $name): int
{
    switch ($name) {
        case 'locator':
            return 1;
        case 'quick test':
            return 2;
    }

    return 0;
}

function switch_on_plain_object_never_matches_string(SwitchOnPlainObject $object): int
{
    switch ($object) {
        // @mago-expect analysis:never-matching-switch-case
        case 'locator':
            return 1;
    }

    return 0;
}
