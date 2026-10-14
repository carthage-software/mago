<?php

const A = 1;
const BBBBB = 2;

$foo = 1;
$barbaz = 2;

$array = [
    'a'    => 1,
    'bbbb' => 2,
    'cc'   => 3,
];

class Example
{
    public const FOO = 1;
    public const BARRRR = 2;

    public string $a = 'a';
    public string $bbbbb = 'b';
}

enum Status: int
{
    case Pending = 0;
    case Approved = 1;
}

function getLabel(Status $status): string
{
    return match ($status) {
        Status::Pending  => 'Pending',
        Status::Approved => 'Approved',
        default          => 'Unknown',
    };
}
