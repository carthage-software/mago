<?php

declare(strict_types=1);

final class GenBaseInst
{
}

/**
 * @template T of GenBaseInst
 *
 * @param class-string<T> $clz
 */
function gen_make(string $clz): GenBaseInst
{
    return new $clz();
}

function take_base(GenBaseInst $b): void
{
}

take_base(gen_make(GenBaseInst::class));
