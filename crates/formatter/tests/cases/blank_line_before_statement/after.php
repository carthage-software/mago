<?php

function demo(bool $flag, iterable $items): Generator
{
    $value = 1;

    /** @var list<int> $numbers */
    $numbers = [1, 2, 3];
    if ($flag) {
        $exception = new RuntimeException();

        throw $exception;
    }

    include 'bootstrap.php';

    yield from $items;
    switch ($value) {
        case 1:
            foo();
            break;

        case 2:
            bar();
            break;

        default:
            $result = count($numbers);

            return $result;
    }
}
