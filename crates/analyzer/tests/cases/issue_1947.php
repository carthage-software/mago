<?php

declare(strict_types=1);

function throwable_chain_1947(Throwable $error): void
{
    do {
        echo $error->getMessage();

        $error = $error->getPrevious();
    } while ($error !== null);
}

function int_chain_1947(int $start): void
{
    $current = $start;

    do {
        echo $current;

        $current = $current > 0 ? $current - 1 : null;
    } while ($current !== null);
}
