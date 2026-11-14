<?php

declare(strict_types=1);

function next_item(): ?string
{
    return rand(0, 1) ? 'x' : null;
}

function firstItem(): string
{
    while (true) {
        $item = next_item();
        if ($item !== null) {
            break;
        }
    }

    return $item;
}

function changedAfterBreak(): int
{
    while (true) {
        $value = 1;
        if (rand(0, 1)) {
            break;
        }

        $value = 'a';
        echo $value;
    }

    return $value;
}
