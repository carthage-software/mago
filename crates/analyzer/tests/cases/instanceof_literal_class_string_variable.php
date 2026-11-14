<?php

declare(strict_types=1);

interface I {}

final class F {}

function literal_interface(I $o): void
{
    $class = I::class;
    // @mago-expect analysis:redundant-condition
    if ($o instanceof $class) {
        echo 'yes';
    }
}

function literal_impossible(F $o): void
{
    $class = I::class;
    // @mago-expect analysis:impossible-condition
    if ($o instanceof $class) {
        echo 'no';
    }
}
