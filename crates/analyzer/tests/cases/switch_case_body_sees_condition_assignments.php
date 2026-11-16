<?php

declare(strict_types=1);

function first_position(string $value): int
{
    switch (true) {
        case ($position = strpos($value, 'a')) !== false:
            return $position;
        case ($position = strpos($value, 'b')) !== false:
            return $position;
    }

    return -1;
}

function length_case(string $value): int
{
    switch (1) {
        case $length = strlen($value):
            return $length;
    }

    return 0;
}

function refill(?int &$value): bool
{
    $value = rand(0, 1) === 1 ? null : 1;

    return true;
}

function by_reference(?int $value): int
{
    switch (true) {
        case $value === null:
            return 0;
        case refill($value) && $value === null:
            return 1;
    }

    return 2;
}

function by_reference_body(?int $value): ?int
{
    switch (true) {
        case $value === null:
            return 0;
        case refill($value):
            return $value;
    }

    return 2;
}
