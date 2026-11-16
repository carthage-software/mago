<?php

declare(strict_types=1);

function condition_narrowing(mixed $value, int $count): void
{
    while ($count < 3 && $value !== null) {
        $count++;
    }

    if ($value === null) {
        echo 'null';
    }
}

function break_narrowing(mixed $value, int $count): void
{
    while ($count < 3) {
        if ($value === null) {
            break;
        }

        $count++;
    }

    if ($value === null) {
        echo 'null';
    }
}

function truthy_for_condition(mixed $value): void
{
    for ($i = 0; $i < 3 && $value; $i++) { // @mago-expect analysis:mixed-operand
        echo $i;
    }

    if (!$value) {
        echo 'falsy';
    }
}
