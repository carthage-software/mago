<?php

enum Color
{
    case Red;
    case Green;
    case Blue;
}

enum ExtendedColor
{
    case Red;
    case Green;
    case Blue;
    case Yellow;
}

enum TextColor
{
    case White;
    case Black;
}

/**
 */
function get_hex_color(Color|ExtendedColor|TextColor $color): string
{
    return match ($color) {
        Color::Red, ExtendedColor::Red => '#FF0000',
        Color::Green, ExtendedColor::Green => '#00FF00',
        Color::Blue => '#0000FF',
        ExtendedColor::Yellow => '#FFFF00',
        TextColor::White => '#FFFFFF',
        TextColor::Black => '#000000',
    };
}
