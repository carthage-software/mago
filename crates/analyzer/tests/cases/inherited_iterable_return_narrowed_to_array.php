<?php

declare(strict_types=1);

namespace InheritedIterableReturnNarrowedToArray;

use stdClass;

abstract class Base
{
    /** @return iterable<int, stdClass> */
    abstract public function items(): iterable;

    /** @return iterable<stdClass> */
    abstract public function values(): iterable;

    /** @return iterable<string, int>|null */
    abstract public function counts(): ?iterable;

    /** @return iterable<string, int>|null */
    abstract public function maybe(): ?iterable;

    /** @return iterable<int, string> */
    abstract public function documented(): iterable;
}

interface Source
{
    /** @return iterable<int, stdClass> */
    public function rows();
}

final class Valid extends Base implements Source
{
    public function items(): array
    {
        return [new stdClass(), new stdClass()];
    }

    public function values(): array
    {
        return ['first' => new stdClass()];
    }

    public function counts(): array
    {
        return ['a' => 1];
    }

    public function maybe(): ?array
    {
        return null;
    }

    /** @return list<non-empty-string> */
    public function documented(): array
    {
        return ['a'];
    }

    public function rows(): array
    {
        return [new stdClass()];
    }
}

final class Invalid extends Base implements Source
{
    public function items(): array
    {
        // @mago-expect analysis:invalid-return-statement
        return [3, 4];
    }

    public function values(): array
    {
        // @mago-expect analysis:invalid-return-statement
        return ['first' => 1];
    }

    public function counts(): array
    {
        // @mago-expect analysis:invalid-return-statement
        return [1 => 'a'];
    }

    public function maybe(): ?array
    {
        // @mago-expect analysis:invalid-return-statement
        return ['a' => 'b'];
    }

    public function documented(): array
    {
        // @mago-expect analysis:invalid-return-statement
        return [1.5];
    }

    public function rows(): array
    {
        // @mago-expect analysis:invalid-return-statement
        return ['x' => new stdClass()];
    }
}

function first_id(Valid $valid): int
{
    foreach ($valid->items() as $id => $_item) {
        return $id;
    }

    return 0;
}

function total(Valid $valid): int
{
    $total = 0;
    foreach ($valid->counts() as $name => $count) {
        $total += $count + strlen($name);
    }

    return $total;
}
