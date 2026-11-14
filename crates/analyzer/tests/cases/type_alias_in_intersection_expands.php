<?php

declare(strict_types=1);

interface A
{
    public function a(): int;
}

interface B {}

/**
 * @psalm-type One = A
 * @psalm-type Either = A|B
 */
abstract class Foo
{
    /** @param One&self $arg */
    protected function alias_on_left(self $arg): int
    {
        return $arg->a();
    }

    /** @param self&One $arg */
    protected function alias_on_right(self $arg): int
    {
        return $arg->a();
    }

    /** @param (Either)&self $arg */
    protected function union_alias(self $arg): void {}

    /** @param (A|B)&self $arg */
    protected function spelled_out(self $arg): void {}
}
