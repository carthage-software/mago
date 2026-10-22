<?php

declare(strict_types=1);

function accept_false_or_int(false|int $value): void {}

function accept_false_or_string(false|string $value): void {}

function accept_true_or_int(true|int $value): void {}

function not_identical_true_bool_int(bool|int $value): void
{
    if ($value !== true) {
        accept_false_or_int($value);
    }

    if (true !== $value) {
        accept_false_or_int($value);
    }
}

function not_identical_true_bool_string(bool|string $value): void
{
    if ($value !== true) {
        accept_false_or_string($value);
    }
}

function not_identical_false_bool_int(bool|int $value): void
{
    if ($value !== false) {
        accept_true_or_int($value);
    }
}
