<?php

declare(strict_types=1);

declare(ticks=1) {
    function tick_handler(): void
    {
    }

    register_tick_function("tick_handler");

    $a = 1;

    if ($a > 0) {
        $a += 2;

        print $a . PHP_EOL;
    }
}
