<?php

declare(strict_types=1);

final class Item
{
    public function name(): string
    {
        return 'item';
    }
}

function takes_string(string $s): void
{
    echo $s;
}

function is_null_check(): void
{
    if (rand(0, 1)) {
        $item = new Item();
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    if (is_null($item)) {
        echo 'unset';
    }
}

function identical_null_check(): void
{
    if (rand(0, 1)) {
        $item = new Item();
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    if ($item === null) {
        echo 'unset';
    }
}

function falsy_check(): void
{
    if (rand(0, 1)) {
        $item = new Item();
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    if (!$item) {
        echo 'unset';
    }
}

function not_null_check(): void
{
    if (rand(0, 1)) {
        $s = 'abc';
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    if ($s !== null) {
        takes_string($s);
    }
}

function truthy_check(): void
{
    if (rand(0, 1)) {
        $item = new Item();
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    if ($item) {
        echo $item->name();
    }
}
