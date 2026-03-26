<?php

// fits on one line
function fits_on_one_line(#[A] int $a, #[B] int $b) {}

// exceeds print width
function function_exceeding_print_width(
    #[A] int $aaaaaaaaaa,
    #[B] int $bbbbbbbbbb,
    #[C] int $cccccccccc,
    #[D] int $dddddddddd,
) {}

// source-broken
function source_broken(
    #[SensitiveParameter] string $password,
    int $mode,
) {}

// source-broken, exceeds print width when collapsed
function source_broken_wide(
    #[SensitiveParameter] string $password,
    #[Assert\Length(min: 8, max: 128, minMessage: 'The password must be at least 8 characters long')]
    string $confirmation,
) {}

// promoted properties
class Foo
{
    public function __construct(
        #[Assert\NotBlank] public string $name,
        #[SensitiveParameter] public string $password,
    ) {}
}

// closure, fits on one line
$fn = function (#[A] int $a, #[B] int $b) use ($c) {};

// closure, exceeds print width
$fn = function (
    #[SensitiveParameter] string $password,
    #[Assert\NotBlank] string $confirmation,
    #[Assert\Email] string $email,
) use ($ctx) {};

// arrow function, fits on one line
$fn = fn(#[A] int $a) => $a;

// arrow function, exceeds print width
$fn = fn(
    #[SensitiveParameter] string $aaaaaaaaaa,
    #[Assert\NotBlank] string $bbbbbbbbbb,
    #[Assert\Email] string $cccccccccc,
) => null;
