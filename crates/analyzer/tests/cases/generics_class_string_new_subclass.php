<?php

declare(strict_types=1);

class GenBaseInst2
{
}

final class GenChildInst2 extends GenBaseInst2
{
    public int $extra = 0;
}

/**
 * @template T of GenBaseInst2
 *
 * @param class-string<T> $clz
 *
 * @return T
 */
function gen_make2(string $clz): GenBaseInst2
{
    /** @mago-expect analysis:unsafe-instantiation */
    /** @var T */
    $obj = new $clz();
    return $obj;
}

function take_child(GenChildInst2 $c): int
{
    return $c->extra;
}

take_child(gen_make2(GenChildInst2::class));
