<?php

declare(strict_types=1);

enum Status
{
    case A;
    case B;
}

enum Sub
{
    case X;
    case Y;
}

function flow_match_match_in_arm(Status $s, Sub $u): string
{
    return match ($s) {
        Status::A => match ($u) {
            Sub::X => 'ax',
            Sub::Y => 'ay',
        },
        Status::B => 'b',
    };
}
