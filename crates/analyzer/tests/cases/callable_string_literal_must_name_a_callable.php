<?php

declare(strict_types=1);

final class A
{
    public static function hello(): string
    {
        return 'hello';
    }
}

/**
 * @param callable-string $name
 */
function call(string $name): mixed
{
    return $name();
}

call('A::hello');
call('\A::hello');
call('strlen');
call('\strlen');
// @mago-expect analysis:possibly-invalid-argument
call('A::missing');
// @mago-expect analysis:possibly-invalid-argument
call('Missing::hello');
