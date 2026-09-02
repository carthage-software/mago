<?php

declare(strict_types=1);

final class Condition
{
    /** @var list<string> */
    public array $identifiers = [];
}

final class A
{
    /** @var Condition[]|null */
    public array|null $where = null;
}

final class B
{
    /** @var Condition[]|null */
    public array|null $where = null;
}

function repro(object $stmt): void
{
    if (!($stmt instanceof A || $stmt instanceof B) || $stmt->where === null) {
        return;
    }

    foreach ($stmt->where as $expr) {
        sink($expr->identifiers);
    }
}

/** @param list<string> $_ */
function sink(array $_): void {}
