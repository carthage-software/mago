<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param class-string<T> $cls
 *
 * @return class-string<T>
 */
function gen_pass_through_cs(string $cls): string
{
    return $cls;
}

final class GenSomeCC
{
}

$v = gen_pass_through_cs(GenSomeCC::class);
echo $v;
