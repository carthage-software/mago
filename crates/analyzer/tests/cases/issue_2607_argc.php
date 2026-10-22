<?php

declare(strict_types=1);

namespace Issue2607Argc;

/** @param positive-int $_count */
function acceptPositiveCount(int $_count): void {}

acceptPositiveCount($argc);

// @mago-expect analysis:invalid-method-access
$argc->foo();

if ($argc === 2) {
    echo 'One argument was supplied.';
}

if ($argc < 2) {
    exit(1);
}

if ($argc !== 3) {
    echo 'Two arguments are required.';
    exit(1);
}

echo $argc;

function checkImportedArgumentCount(): void
{
    global $argc;

    acceptPositiveCount($argc);

    if ($argc === 1) {
        echo 'No arguments were supplied.';
    }

    if ($argc >= 3) {
        echo 'At least two arguments were supplied.';
    }
}
