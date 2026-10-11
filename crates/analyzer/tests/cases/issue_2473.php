<?php

/**
 * @template T
 * @implements ArrayAccess<int, T>
 */
final class Rows implements ArrayAccess
{
    /** @var array<int, mixed> */
    private array $items = [];

    #[Override]
    public function offsetExists(mixed $offset): bool
    {
        return isset($this->items[$offset]);
    }

    #[Override]
    public function offsetGet(mixed $offset): mixed
    {
        // @mago-expect analysis:mixed-return-statement
        return $this->items[$offset];
    }

    /**
     * @param int|null $offset
     * @param mixed $value
     */
    #[Override]
    public function offsetSet(mixed $offset, mixed $value): void
    {
        $this->items[] = $value;
    }

    #[Override]
    public function offsetUnset(mixed $offset): void
    {
        unset($this->items[$offset]);
    }
}

/** @param Rows<array<string, mixed>> $rows */
function add(Rows $rows): void
{
    $rows->offsetSet(null, 5);
    $rows[] = 5;
    $rows[0] = 5;
}

/**
 * @template T
 * @implements ArrayAccess<int, T>
 */
class TypedRows implements ArrayAccess
{
    /** @var array<int, T> */
    private array $items = [];

    #[Override]
    public function offsetExists(mixed $offset): bool
    {
        return isset($this->items[$offset]);
    }

    #[Override]
    public function offsetGet(mixed $offset): mixed
    {
        return $this->items[$offset];
    }

    /**
     * @param int|null $offset
     * @param T $value
     */
    #[Override]
    public function offsetSet(mixed $offset, mixed $value): void
    {
        $this->items[] = $value;
    }

    #[Override]
    public function offsetUnset(mixed $offset): void
    {
        unset($this->items[$offset]);
    }
}

/**
 * @template T
 * @extends TypedRows<T>
 */
final class InheritedTypedRows extends TypedRows {}

/**
 * @template T
 * @extends TypedRows<T>
 */
final class InheritedValueRows extends TypedRows
{
    /** @param int|null $offset */
    #[Override]
    public function offsetSet(mixed $offset, mixed $value): void {}
}

/**
 * @template T
 * @extends TypedRows<T>
 */
class MixedRows extends TypedRows
{
    /**
     * @param int|null $offset
     * @param mixed $value
     */
    #[Override]
    public function offsetSet(mixed $offset, mixed $value): void {}
}

/**
 * @template T
 * @extends MixedRows<T>
 */
final class InheritedMixedRows extends MixedRows {}

/**
 * @template T
 * @extends TypedRows<list<T>>
 */
final class NestedTypedRows extends TypedRows {}

/** @param TypedRows<array<string, mixed>> $rows */
function addTyped(TypedRows $rows): void
{
    $rows->offsetSet(null, ['name' => 'example']);
    $rows[] = ['name' => 'example'];
    $rows[0] = ['name' => 'example'];

    // @mago-expect analysis:invalid-argument
    $rows->offsetSet(null, 5);
    // @mago-expect analysis:invalid-array-access-assignment-value
    $rows[] = 5;
    // @mago-expect analysis:invalid-array-access-assignment-value
    $rows[0] = 5;
}

/** @param InheritedTypedRows<array<string, mixed>> $rows */
function addInheritedTyped(InheritedTypedRows $rows): void
{
    $rows[] = ['name' => 'example'];
    // @mago-expect analysis:invalid-array-access-assignment-value
    $rows[] = 5;
}

/** @param InheritedValueRows<array<string, mixed>> $rows */
function addInheritedValue(InheritedValueRows $rows): void
{
    $rows[] = ['name' => 'example'];
    // @mago-expect analysis:invalid-argument
    $rows->offsetSet(null, 5);
    // @mago-expect analysis:invalid-array-access-assignment-value
    $rows[] = 5;
}

/** @param MixedRows<array<string, mixed>> $rows */
function addMixed(MixedRows $rows): void
{
    $rows->offsetSet(null, 5);
    $rows[] = 5;
    $rows[0] = 5;
}

/** @param InheritedMixedRows<array<string, mixed>> $rows */
function addInheritedMixed(InheritedMixedRows $rows): void
{
    $rows->offsetSet(null, 5);
    $rows[] = 5;
    $rows[0] = 5;
}

/** @param NestedTypedRows<string> $rows */
function addNested(NestedTypedRows $rows): void
{
    $rows[] = ['example'];
    // @mago-expect analysis:invalid-array-access-assignment-value
    $rows[] = [5];
}

/** @param ArrayAccess<int, array<string, mixed>> $rows */
function addInterface(ArrayAccess $rows): void
{
    $rows[0] = ['name' => 'example'];
    // @mago-expect analysis:invalid-array-access-assignment-value
    $rows[0] = 5;
}

/**
 * @param Rows<array<string, mixed>> $rows
 * @return array<string, mixed>
 */
function readRow(Rows $rows): array
{
    return $rows[0];
}
