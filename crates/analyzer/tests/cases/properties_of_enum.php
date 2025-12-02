<?php

enum Color: string
{
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';
}

enum Status: int
{
    case Pending = 0;
    case Active = 1;
    case Inactive = 2;
}

enum Direction
{
    case North;
    case South;
    case East;
    case West;
}

/**
 * @param array{name: 'Red'|'Green'|'Blue', value: 'red'|'green'|'blue'} $_
 */
function accepts_color_properties(array $_): void
{
}

/**
 * @param array{name: 'Pending'|'Active'|'Inactive', value: 0|1|2} $_
 */
function accepts_status_properties(array $_): void
{
}

/**
 * @param array{name: 'North'|'South'|'East'|'West'} $_
 */
function accepts_direction_properties(array $_): void
{
}

/**
 * @return properties-of<Color>
 */
function get_color_properties(Color $c): array
{
    return ['name' => $c->name, 'value' => $c->value];
}

/**
 * @return properties-of<Status>
 */
function get_status_properties(Status $s): array
{
    return ['name' => $s->name, 'value' => $s->value];
}

/**
 * @return properties-of<Direction>
 */
function get_direction_properties(Direction $d): array
{
    return ['name' => $d->name];
}

function test_backed_enum_properties(): void
{
    $props = get_color_properties(Color::Red);
    accepts_color_properties($props);
}

function test_int_backed_enum_properties(): void
{
    $props = get_status_properties(Status::Active);
    accepts_status_properties($props);
}

function test_unit_enum_properties(): void
{
    $props = get_direction_properties(Direction::North);
    accepts_direction_properties($props);
}
