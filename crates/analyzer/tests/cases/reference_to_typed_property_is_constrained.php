<?php

declare(strict_types=1);

final class ReferencedTypedProperties
{
    public int $count = 0;
    public static int $shared = 0;
    /**
     * @var string|int
     */
    public $loose = '';
}

function reference_instance_property(ReferencedTypedProperties $properties): void
{
    $count = &$properties->count;
    $count = 5;

    /** @mago-expect analysis:reference-constraint-violation */
    $count = '';
}

function reference_static_property(): void
{
    $shared = &ReferencedTypedProperties::$shared;

    /** @mago-expect analysis:reference-constraint-violation */
    $shared = '';
}

function reference_untyped_property(ReferencedTypedProperties $properties): void
{
    $loose = &$properties->loose;
    $loose = 1;
}

function rebind_reference(ReferencedTypedProperties $properties, string $value): void
{
    $count = &$properties->count;
    $count = &$value;
    $count = 'rebound';
}
