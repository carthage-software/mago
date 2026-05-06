<?php

declare(strict_types=1);

enum Direction
{
    case North;
    case South;
    case East;
    case West;
}

function flow_match_exhaustive_enum(Direction $d): string
{
    return match ($d) {
        Direction::North => 'n',
        Direction::South => 's',
        Direction::East => 'e',
        Direction::West => 'w',
    };
}
