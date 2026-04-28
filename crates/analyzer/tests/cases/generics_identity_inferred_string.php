<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 *
 * @return T
 */
function gen_id_string(mixed $value): mixed
{
    return $value;
}

function takes_int(int $n): void
{
}

/** @mago-expect analysis:invalid-argument */
takes_int(gen_id_string('hello'));
