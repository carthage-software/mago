<?php

declare(strict_types=1);

/**
 * @param object{name: ?string} $obj
 *
 * @psalm-assert-if-true !null $obj->name
 */
function has_name(object $obj): bool
{
    return $obj->name !== null;
}

/**
 * @param object{name: ?string} $user
 */
function name_of(object $user): string
{
    if (has_name($user)) {
        return $user->name;
    }

    return '';
}
