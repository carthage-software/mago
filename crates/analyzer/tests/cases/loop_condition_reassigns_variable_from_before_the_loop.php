<?php

declare(strict_types=1);

function takes_false(false $f): void
{
    var_dump($f);
}

function takes_int(int $i): void
{
    echo $i;
}

function read_lines(): void
{
    $line = '';
    while (false !== ($line = fgets(STDIN))) {
        echo $line;
    }

    takes_false($line);
}

function drain(?int $v): void
{
    while ($v = rand(0, 3)) {
        echo 'again';
    }

    takes_int($v);
}

function flip(): void
{
    $v = true;
    while ($v = rand(0, 1) === 1) {
        echo 'again';
    }

    takes_false($v);
}

function several_conditions(): void
{
    $value = 'a';
    for (; $value = 42, $value = rand(0, 1) === 1;) {
        echo 'again';
    }

    takes_false($value);
}

/** @return 0 */
function last_value(): int
{
    $result = 1;
    while ($result = rand(0, 6)) {
        echo 'again';
    }

    return $result;
}
