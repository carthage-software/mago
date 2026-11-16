<?php

declare(strict_types=1);

function infinite_loop(): string
{
    while (true) {
        if (rand(0, 1) === 1) {
            break;
        }

        $car = 'Volvo';
        break;
    }

    return $car; // @mago-expect analysis:possibly-undefined-variable
}

function bounded_for_loop(): string
{
    for ($i = 0; $i < 4; $i++) {
        if (rand(0, 1) === 1) {
            break;
        }

        $car = 'Volvo';
    }

    return $car; // @mago-expect analysis:possibly-undefined-variable
}

function assigned_before_every_break(): string
{
    while (true) {
        $car = 'Volvo';
        if (rand(0, 1) === 1) {
            break;
        }
    }

    return $car;
}
