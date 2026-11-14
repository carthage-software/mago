<?php

declare(strict_types=1);

final class Name
{
}

final class Row
{
    public const string Name = 'constant';

    public int $Name = 1;
}

function property_by_class_constant(Row $row): int
{
    return $row->{Name::class};
}

function constant_by_class_constant(): string
{
    return Row::{Name::class};
}

function missing_property(Row $row): mixed
{
    /** @mago-expect analysis:non-existent-property */
    return $row->{Row::class};
}
