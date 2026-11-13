<?php

declare(strict_types=1);

interface Marker {}

class Base {}

class Child extends Base {}

final class FinalChild extends Base {}

final class MarkedChild extends Base implements Marker {}

function child_after_marker(Base $value): ?Child
{
    if ($value instanceof Marker && $value instanceof Child) {
        return $value;
    }

    return null;
}

function marked_child_after_marker(Base $value): ?MarkedChild
{
    if ($value instanceof Marker && $value instanceof MarkedChild) {
        return $value;
    }

    return null;
}

function final_child_after_marker(Base $value): void
{
    if ($value instanceof Marker) {
        // @mago-expect analysis:impossible-condition
        if ($value instanceof FinalChild) {
            echo 'unreachable';
        }
    }
}

final class Box
{
    /** @psalm-assert-if-true int|string $this->get() */
    public function isScalar(): bool
    {
        return true;
    }

    public function get(): mixed
    {
        return 1;
    }
}

function scalar_of(Box $box): int|string
{
    if ($box->isScalar()) {
        return $box->get();
    }

    return 0;
}
