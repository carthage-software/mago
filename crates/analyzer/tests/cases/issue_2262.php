<?php

declare(strict_types=1);

final class Child
{
    public function __construct(public int $id) {}
}

final class Container
{
    public function __construct(public ?Child $child) {}

    /** @phpstan-assert-if-true !null $this->child */
    public function hasChild(): bool
    {
        return $this->child !== null;
    }

    /** @phpstan-assert-if-false !null $this->child */
    public function lacksChild(): bool
    {
        return $this->child === null;
    }

    /** @phpstan-assert-if-true !null $this->child */
    public function maybeHasChild(): ?bool
    {
        return rand(0, 1) === 1 ? null : $this->child !== null;
    }
}

function trueAssertion(Container $container): int
{
    return match ($container->hasChild()) {
        true => $container->child->id,
        false => 0,
    };
}

function falseAssertion(Container $container): int
{
    return match ($container->lacksChild()) {
        false => $container->child->id,
        true => 0,
    };
}

function explicitComparisons(Container $container): int
{
    if ($container->hasChild() === true) {
        return $container->child->id;
    }

    if ($container->hasChild() !== false) {
        return $container->child->id;
    }

    return 0;
}

function nullableBooleanAssertion(Container $container): int
{
    return match ($container->maybeHasChild()) {
        true => $container->child->id,
        default => 0,
    };
}

function nonExactComparison(Container $container): ?int
{
    if ($container->maybeHasChild() !== false) {
        // @mago-expect analysis:possibly-null-property-access
        return $container->child->id;
    }

    return null;
}

function ifAssertions(Container $container): int
{
    if ($container->hasChild()) {
        return $container->child->id;
    }

    if (!$container->lacksChild()) {
        return $container->child->id;
    }

    return 0;
}
