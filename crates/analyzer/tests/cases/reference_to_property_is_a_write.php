<?php

declare(strict_types=1);

final class ReferencedProperties
{
    public private(set) int $guarded = 0;
    public readonly int $fixed;
    public int $open = 0;

    /**
     * @var array<string, list<int>>
     */
    private array $groups = [];

    public function __construct()
    {
        $this->fixed = 1;
    }

    public function group(string $key, int $item): void
    {
        $group = &$this->groups[$key];
        $group[] = $item;
    }

    public function bump(): void
    {
        $guarded = &$this->guarded;
        $guarded++;
    }
}

function reference_guarded(ReferencedProperties $properties): void
{
    /** @mago-expect analysis:invalid-property-write */
    $guarded = &$properties->guarded;
    echo $guarded;
}

function reference_fixed(ReferencedProperties $properties): void
{
    /** @mago-expect analysis:invalid-property-write */
    $fixed = &$properties->fixed;
    echo $fixed;
}

function reference_open(ReferencedProperties $properties): void
{
    $open = &$properties->open;
    $open++;
}
