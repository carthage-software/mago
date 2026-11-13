<?php

declare(strict_types=1);

namespace Issue2629;

final class Item
{
    public function id(): int
    {
        return 1;
    }
}

abstract class Base
{
    public function __construct(
        public readonly ?Item $item,
        public ?Item $mutable = null,
    ) {}

    public function ping(): void {}
}

final class Child extends Base {}

trait HasItem
{
    public readonly ?Item $item;

    public function ping(): void {}
}

final class WithTrait
{
    use HasItem;

    public function __construct(?Item $item)
    {
        $this->item = $item;
    }
}

final class Asymmetric
{
    public function __construct(
        public private(set) ?Item $item,
    ) {}

    public function ping(): void
    {
        $this->item = null;
    }
}

final class Holder
{
    public function __construct(
        public readonly Child $child,
        public Child $other,
    ) {}

    public function ping(): void {}
}

final class Service
{
    private readonly ?Item $item;
    private ?Item $cache = null;

    public function __construct(?Item $item)
    {
        $this->item = $item;
    }

    public function viaThisReadonly(): int
    {
        if ($this->item === null) {
            return 0;
        }

        $this->touch();

        return $this->item->id();
    }

    public function viaThisMutable(): int
    {
        if ($this->cache === null) {
            return 0;
        }

        $this->touch();

        // @mago-expect analysis:possible-method-access-on-null
        return $this->cache->id();
    }

    private function touch(): void
    {
        $this->cache = null;
    }
}

function consume(object $object): void {}

function inherited(Child $object): int
{
    if ($object->item === null) {
        return 0;
    }

    $object->ping();

    return $object->item->id();
}

function inherited_mutable(Child $object): int
{
    if ($object->mutable === null) {
        return 0;
    }

    $object->ping();

    // @mago-expect analysis:possible-method-access-on-null
    return $object->mutable->id();
}

function from_trait(WithTrait $object): int
{
    if ($object->item === null) {
        return 0;
    }

    $object->ping();

    return $object->item->id();
}

function asymmetric(Asymmetric $object): int
{
    if ($object->item === null) {
        return 0;
    }

    $object->ping();

    // @mago-expect analysis:possible-method-access-on-null
    return $object->item->id();
}

function nested_readonly(Holder $holder): int
{
    if ($holder->child->item === null) {
        return 0;
    }

    $holder->ping();

    return $holder->child->item->id();
}

function nested_through_mutable(Holder $holder): int
{
    if ($holder->other->item === null) {
        return 0;
    }

    $holder->ping();

    // @mago-expect analysis:possible-method-access-on-null
    return $holder->other->item->id();
}

function passed_as_argument(Child $object): int
{
    if ($object->item === null) {
        return 0;
    }

    consume($object);

    return $object->item->id();
}

function nullsafe_call(?Child $object): int
{
    if ($object?->item === null) {
        return 0;
    }

    $object->ping();

    return $object->item->id();
}

function union_receiver(Child|WithTrait $object): int
{
    if ($object->item === null) {
        return 0;
    }

    $object->ping();

    return $object->item->id();
}

function union_with_mutable(Child|Asymmetric $object): int
{
    if ($object->item === null) {
        return 0;
    }

    $object->ping();

    // @mago-expect analysis:possible-method-access-on-null
    return $object->item->id();
}

final class BodyReadonly
{
    public readonly ?Item $item;

    public function __construct(?Item $item)
    {
        $this->item = $item;
    }

    public function ping(): void {}
}

final readonly class ReadonlyClass
{
    public function __construct(
        public ?Item $item,
    ) {}

    public function ping(): void {}
}

function body_readonly(BodyReadonly $object): int
{
    if ($object->item === null) {
        return 0;
    }

    $object->ping();

    return $object->item->id();
}

function readonly_class(ReadonlyClass $object): int
{
    if ($object->item === null) {
        return 0;
    }

    $object->ping();

    return $object->item->id();
}

function other_object(Child $object, Child $other): int
{
    if ($object->item === null) {
        return 0;
    }

    $other->ping();

    return $object->item->id();
}
