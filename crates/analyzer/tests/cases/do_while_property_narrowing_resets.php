<?php

declare(strict_types=1);

final class DoWhileOptions
{
    public ?string $s = null;
}

function do_while_property_0(DoWhileOptions $options): void
{
    $i = 0;
    do {
        if ($options->s === null) {
            echo 'null';
        }

        $i++;
    } while ($options->s !== null && $i < 3);
}

function do_while_property_1(DoWhileOptions $options): void
{
    $i = 0;
    do {
        if ($options->s === null) {
            echo 'null';
        }

        $options->s = null;
        $i++;
    } while (false);
}
