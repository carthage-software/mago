<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

function example(bool $b): string {
    return match ($b) {
        true => 'yes',
        false => 'no',
    };
}

takesString(example(true));
