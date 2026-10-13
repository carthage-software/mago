<?php

declare(strict_types=1);

namespace App;

/**
 * @param list<string> $ids
 */
function report(array $ids): void
{
    /** @var string $id */
    foreach ($ids as $idRaw) {
        echo $id; // @mago-expect analysis:undefined-variable, analysis:mixed-argument
    }
}

function inline_var_unassigned(): string
{
    /** @var string $name */
    return $name; // @mago-expect analysis:undefined-variable, analysis:mixed-return-statement
}

function inline_var_defined_is_narrowed(): string
{
    $name = "test";
    /** @var non-empty-string $name */
    return $name;
}
