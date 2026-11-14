<?php

declare(strict_types=1);

final class Pos
{
    /** @var positive-int */
    public int $id = 1;
}

final class Wide
{
    public int|string $id = 'x';
}

/**
 * @property int $id
 */
final class NoGetter {}

/**
 * @property int $id
 */
final class WithGetter
{
    public function __get(string $name): int
    {
        return strlen($name);
    }
}

final class Priv
{
    private int $id = 1;

    public function id(): int
    {
        return $this->id;
    }
}

/**
 * @param object{id: int, ...} $object
 */
function read_id(object $object): int
{
    return $object->id;
}

function test(): void
{
    echo read_id(new Pos());
    echo read_id(new WithGetter());
    // @mago-expect analysis:invalid-argument
    echo read_id(new Wide());
    // @mago-expect analysis:invalid-argument
    echo read_id(new NoGetter());
    // @mago-expect analysis:invalid-argument
    echo read_id(new Priv());
}
