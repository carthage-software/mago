<?php

declare(strict_types=1);

class ParentInTraitBase
{
    public function boot(): void
    {
    }

    public static function make(): void
    {
    }
}

trait ParentInTraitBoots
{
    public function boot(): void
    {
        parent::boot();
    }

    public static function make(): void
    {
        parent::make();
    }
}

final class ParentInTraitChild extends ParentInTraitBase
{
    use ParentInTraitBoots;
}

final class ParentInTraitOrphan
{
    public function boot(): void
    {
        // @mago-expect analysis:invalid-parent-type
        parent::boot();
    }
}
