<?php

declare(strict_types=1);

class MyBase
{
}

final class MyTest extends MyBase
{
    public int $i = 0;
}

/**
 * @template T of MyBase
 * @param class-string<T> $className
 * @return T
 */
function create_object(string $className): object
{
    /** @mago-expect analysis:unsafe-instantiation */
    return new $className();
}

function consume_class_const(): int
{
    $c = create_object(MyTest::class);
    return $c->i;
}

function consume_literal_string(): int
{
    $c = create_object('MyTest');
    return $c->i;
}
