<?php

declare(strict_types=1);

namespace Issue2250;

final class Value
{
    public function __construct(public int $amount) {}
}

final class Container
{
    public function __construct(public Value $tagValue) {}

    public function getTagValue(): Value
    {
        return $this->tagValue;
    }
}

final class Outer
{
    public function __construct(public ?Container $container) {}
}

function propertyAccess(?Container $container): bool
{
    return $container?->tagValue instanceof Value && $container->tagValue->amount > 0;
}

function methodCall(?Container $container): bool
{
    return $container?->getTagValue() instanceof Value && $container->tagValue->amount > 0;
}

/** @mago-expect analysis:possibly-null-property-access */
function logicalOr(?Container $container): bool
{
    return $container?->tagValue instanceof Value || $container->tagValue instanceof Value;
}

function nestedAccess(?Outer $outer): bool
{
    return $outer?->container?->tagValue instanceof Value && $outer->container->tagValue->amount > 0;
}

/** @mago-expect analysis:possibly-null-property-access */
function negatedBranch(?Container $container): void
{
    if ($container?->tagValue instanceof Value) {
        return;
    }

    $amount = $container->tagValue->amount;
}

function guardClause(?Container $container): int
{
    if (!($container?->tagValue instanceof Value)) {
        return 0;
    }

    return $container->tagValue->amount;
}

/** @mago-expect analysis:possibly-null-property-access */
function falseComparison(?Container $container): void
{
    if (($container?->tagValue instanceof Value) === false) {
        $amount = $container->tagValue->amount;
    }
}

function notFalseComparison(?Container $container): void
{
    if (($container?->tagValue instanceof Value) !== false) {
        $amount = $container->tagValue->amount;
    }
}
