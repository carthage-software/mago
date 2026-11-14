<?php

declare(strict_types=1);

function next_item(): ?string
{
    return rand(0, 1) ? 'x' : null;
}

function waitFor(): string
{
    while (($item = next_item()) === null) {
        usleep(10);
    }

    return $item;
}

function retry(): string
{
    while (!($item = next_item())) {
        usleep(10);
    }

    return $item;
}

/** @param resource $handle */
function drain($handle): bool
{
    while (($row = fgets($handle)) !== false) {
        echo $row;
    }

    return $row;
}
