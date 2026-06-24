<?php

declare(strict_types=1);

function print_x_times(string $message, int $times = 1): void
{
    for ($i = 0; $i < $times; $i++) {
        print $message;
    }
}

/** @mago-expect analysis:too-few-arguments */
print_x_times(times: 10);
print_x_times(message: 'hello');
print_x_times(message: 'hello', times: 5);
print_x_times('hello', times: 5);
print_x_times('hello');
