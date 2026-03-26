<?php declare(strict_types=1);

function test(string ...$names): void
{
    /** @var list<string> */
    $bar = [];
    foreach ($names as $name) {
        $bar[] = 'a';
    }
    if (!count($bar)) {
        echo 'z';
        return;
    }

    if (count($bar) === 1) {
        echo 'y';
        return;
    } else {
        echo 'x';
        return;
    }
}
