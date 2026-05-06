<?php

declare(strict_types=1);

/** @var array{ foo?: string } $x */
$x = [];

if (!isset($x['foo'])) {
    echo $x['foo'];
}

if (isset($x['foo'])) {
    // This is correctly diagnosed as always being a string
    echo $x['foo'];
} else {
    echo $x['foo'];
}
