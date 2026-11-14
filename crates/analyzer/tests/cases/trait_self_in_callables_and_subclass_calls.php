<?php

declare(strict_types=1);

/**
 * @template TOut
 * @param callable(): TOut $c
 * @return TOut
 */
function trait_self_apply(callable $c): mixed
{
    return $c();
}

trait TraitSelfMaker
{
    public static function make(): self
    {
        return new self();
    }

    public function me(): static
    {
        return $this;
    }

    public function build(): self
    {
        return new self();
    }
}

class TraitSelfUser
{
    use TraitSelfMaker;

    public function hello(): void {}
}

final class TraitSelfChild extends TraitSelfUser
{
    public function childOnly(): void {}
}

function trait_self_callables(): void
{
    trait_self_apply([TraitSelfUser::class, 'make'])->hello();
    trait_self_apply('TraitSelfUser::make')->hello();
    trait_self_apply(TraitSelfUser::make(...))->hello();
    trait_self_apply([new TraitSelfChild(), 'me'])->childOnly();
}

function trait_self_subclass_calls(): void
{
    (new TraitSelfChild())->build()->hello();
    // @mago-expect analysis:non-existent-method
    (new TraitSelfChild())->build()->childOnly();
    // @mago-expect analysis:non-existent-method
    TraitSelfChild::make()->childOnly();
}
