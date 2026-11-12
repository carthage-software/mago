<?php

declare(strict_types=1);

namespace FlowElseifReassignsNarrowedVariable;

function elseifAssigns(?int $value, int $i): int
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        $value = 'str';
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function elseifAssignsWithoutNull(?int $value, int $i): int|string
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        $value = 'str';
    }

    return $value;
}

function elseifAssignsBeforeElse(?int $value, int $i): int
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        $value = 'str';
    } else {
        echo 'x';
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function elseAssigns(?int $value, int $i): int
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        echo 'x';
    } else {
        $value = 'str';
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function secondElseifAssigns(?int $value, int $i): int|string
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        $value = 'str';
    } elseif ($i > 2) {
        $value = 1.5;
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function elseifAssignsAfterReturningIf(?int $value, int $i): int
{
    if ($value === null) {
        return 0;
    } elseif ($i > 3) {
        $value = 'str';
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function elseAssignsAfterReturningIf(?int $value, int $i): int
{
    if ($value === null) {
        return 0;
    } elseif ($i > 3) {
        echo 'x';
    } else {
        $value = 'str';
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function elseifAssignsNull(?int $value, int $i): int
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        $value = null;
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function nullCheckAfterElseifAssignsNull(?int $value, int $i): string
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        $value = null;
    }

    if ($value === null) {
        return 'null';
    }

    return 'int';
}

function elseifAssignsInNestedIf(?int $value, int $i): int
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        if ($i > 10) {
            $value = 'str';
        }
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

/** @param-out string $out */
function fill(mixed &$out): void
{
    $out = 'x';
}

function elseifAssignsByReference(?int $value, int $i): int
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        fill($value);
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function elseifAssignsInLoop(?int $value, int $i): ?int
{
    while ($i-- > 0) {
        if ($value === null) {
            $value = 5;
        } elseif ($i > 3) {
            $value = 'str';
        }
    }

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function elseifNarrowsThenAssigns(int|string|null $value): float|int
{
    if ($value === null) {
        $value = 5;
    } elseif (\is_string($value)) {
        $value = 1.5;
    }

    return $value;
}

function elseifAssignsSameType(?int $value, int $i): int
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        $value = 7;
    }

    return $value;
}

function elseifDoesNotAssign(?int $value, int $i): int
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        echo 'x';
    } else {
        echo 'y';
    }

    return $value;
}

/** @return 'str'|1.5|5 */
function everyBranchAssigns(?int $value, int $i): float|int|string
{
    if ($value === null) {
        $value = 5;
    } elseif ($i > 3) {
        $value = 'str';
    } else {
        $value = 1.5;
    }

    return $value;
}
