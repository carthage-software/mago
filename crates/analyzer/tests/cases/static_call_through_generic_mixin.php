<?php

declare(strict_types=1);

/** @template T */
final class Repository
{
    /** @return T|null */
    public function first(): mixed
    {
        return null;
    }
}

final class Labels
{
    public function label(): string
    {
        return '';
    }
}

/**
 * @mixin Labels
 * @mixin Repository<static>
 */
abstract class Model
{
    /** @param list<mixed> $args */
    public function __call(string $name, array $args): mixed
    {
        return null;
    }

    /** @param list<mixed> $args */
    public static function __callStatic(string $name, array $args): mixed
    {
        return null;
    }
}

final class Child extends Model {}

/** @mixin Repository<int> */
final class Comment
{
    /** @param list<mixed> $args */
    public static function __callStatic(string $name, array $args): mixed
    {
        return null;
    }
}

function first_child(): ?Child
{
    return Child::first();
}

function first_comment(): ?int
{
    return Comment::first();
}

function child_label(): string
{
    return Child::label();
}
