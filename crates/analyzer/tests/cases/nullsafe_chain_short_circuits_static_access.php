<?php

declare(strict_types=1);

final class NullsafeStaticChain
{
    public static int $count = 1;

    public ?NullsafeStaticChain $maybe = null;

    public function next(): NullsafeStaticChain
    {
        return $this;
    }

    public static function make(): int
    {
        return 1;
    }
}

function nullsafe_chain_static_method(?NullsafeStaticChain $chain): ?int
{
    return $chain?->next()::make();
}

function nullsafe_chain_static_property(?NullsafeStaticChain $chain): ?int
{
    return $chain?->next()::$count;
}

function nullsafe_chain_nullable_member(?NullsafeStaticChain $chain): ?int
{
    /** @mago-expect analysis:possibly-null-property-access */
    return $chain?->maybe::$count;
}
