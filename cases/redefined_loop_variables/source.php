<?php

/**
 * @param list<int> $data
 */
function test_1(array $data): void
{
    $success = false;
    foreach ($data as $_item) {
        $success = true;
    }

    if ($success) {
        echo 'Success!';
    } else {
        echo 'Failure!';
    }
}

/**
 * @param list<int> $data
 */
function test_2(array $data): void
{
    $success = false;
    foreach ($data as $_item) {
        $success = true;
        break;
    }

    if ($success) {
        echo 'Success!';
    } else {
        echo 'Failure!';
    }
}

/**
 * @param list<int> $data
 */
function test_3(array $data): void
{
    $success = false;
    for (;;) {
        $item = $data[0] ?? null;
        if ($item === null) {
            break;
        }

        $success = true;
        break;
    }

    if ($success) {
        echo 'Success!';
    } else {
        echo 'Failure!';
    }
}

/**
 * @param list<int> $data
 */
function test_4(array $data): void
{
    $success = false;
    do {
        $item = $data[0] ?? null;
        if ($item === null) {
            break;
        }

        $success = true;
        break;
    } while (true);

    if ($success) {
        echo 'Success!';
    } else {
        echo 'Failure!';
    }
}

/**
 * @param list<int> $data
 */
function test_5(array $data): void
{
    $success = false;
    while (true) {
        $item = $data[0] ?? null;
        if ($item === null) {
            break;
        }

        $success = true;
        break;
    }

    if ($success) {
        echo 'Success!';
    } else {
        echo 'Failure!';
    }
}
