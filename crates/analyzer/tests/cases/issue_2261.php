<?php

declare(strict_types=1);

namespace Issue2261;

use InvalidArgumentException;

final class Node
{
    public function __construct(public object $marker) {}

    public function getMarker(): object
    {
        return $this->marker;
    }
}

final class Wrapper
{
    public function __construct(public ?Node $node) {}
}

final class NullableNode
{
    public function __construct(public ?object $marker) {}
}

final class Repro
{
    public function inlineNotNullGuard(?Node $node): ?object
    {
        return $node?->marker !== null ? $node->marker : null;
    }

    /** @throws InvalidArgumentException */
    public function nullGuardThenUse(?Node $node): object
    {
        if ($node?->marker === null) {
            throw new InvalidArgumentException();
        }

        return $node->marker;
    }

    public function extractedLocalGuard(?Node $node): ?object
    {
        $marker = $node?->marker;

        if ($marker !== null) {
            return $node->marker;
        }

        return null;
    }

    /** @throws InvalidArgumentException */
    public function nullMethodGuardThenUse(?Node $node): object
    {
        if ($node?->getMarker() === null) {
            throw new InvalidArgumentException();
        }

        return $node->getMarker();
    }

    public function extractedMethodGuard(?Node $node): ?object
    {
        $marker = $node?->getMarker();

        if ($marker !== null) {
            return $node->getMarker();
        }

        return null;
    }

    /** @throws InvalidArgumentException */
    public function nestedGuard(?Wrapper $wrapper): object
    {
        if ($wrapper?->node?->marker === null) {
            throw new InvalidArgumentException();
        }

        return $wrapper->node->marker;
    }

    public function nestedExtractedGuard(?Wrapper $wrapper): ?object
    {
        $marker = $wrapper?->node?->marker;

        if ($marker !== null) {
            return $wrapper->node->marker;
        }

        return null;
    }

    public function nullableProperty(?NullableNode $node): ?object
    {
        if ($node?->marker !== null) {
            return $node->marker;
        }

        return null;
    }

    /** @mago-expect analysis:possibly-null-property-access */
    public function nullBranchStaysNullable(?Node $node): ?object
    {
        if ($node?->marker === null) {
            return $node->marker;
        }

        return null;
    }

    /** @mago-expect analysis:possible-method-access-on-null */
    public function nullMethodBranchStaysNullable(?Node $node): ?object
    {
        if ($node?->getMarker() === null) {
            $node->getMarker();
        }

        return null;
    }

    /** @mago-expect analysis:null-property-access */
    public function reassignedReceiver(?Node $node): ?object
    {
        $marker = $node?->marker;
        $node = null;

        if ($marker !== null) {
            return $node->marker;
        }

        return null;
    }
}

enum Status
{
    case Passed;
    case Failed;
}

final class Payload
{
    public function __construct(
        public Status $status,
        public ?object $details,
    ) {}

    public function getStatus(): Status
    {
        return $this->status;
    }
}

function evaluate(?Payload $container): bool
{
    return match ($container?->status) {
        Status::Passed => true,
        Status::Failed => $container->details !== null,
        default => false,
    };
}

function evaluateMethod(?Payload $container): bool
{
    return match ($container?->getStatus()) {
        Status::Passed => true,
        Status::Failed => $container->details !== null,
        default => false,
    };
}

/** @mago-expect analysis:possibly-null-property-access */
function evaluateNullArm(?Payload $container): ?object
{
    return match ($container?->status) {
        null => $container->details,
        default => null,
    };
}

/** @mago-expect analysis:possibly-null-property-access */
function evaluateMixedArm(?Payload $container): ?object
{
    return match ($container?->status) {
        Status::Passed, null => $container->details,
        default => null,
    };
}
