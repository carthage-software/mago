<?php

declare(strict_types=1);

/**
 * @param list<string> $foo
 *
 * @throws Exception
 */
function pick(array $foo, int $amount): void
{
    if ($amount > count($foo)) {
        throw new Exception('Cannot pick more than we have');
    }

    if ($amount === 1) {
        echo 'amount is one, foo contains items!';
    }
}
