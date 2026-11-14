<?php

declare(strict_types=1);

/** @param list<string|int> $l */
function takes_list(array $l): void
{
}

/**
 * @param array{0?: string, 2: int} $gap
 * @param array{0?: string, 1: int} $optional_first
 * @param array{0: string, 1?: int} $optional_last
 * @param array{0: string, 1: int} $complete
 */
function f(array $gap, array $optional_first, array $optional_last, array $complete): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_list($gap);
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_list($optional_first);
    takes_list($optional_last);
    takes_list($complete);
}
