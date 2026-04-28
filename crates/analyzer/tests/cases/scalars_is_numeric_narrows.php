<?php

declare(strict_types=1);

function example(mixed $x): bool {
    return is_numeric($x);
}

example('1.5');
example(5);
example('foo');
