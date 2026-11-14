<?php

declare(strict_types=1);

interface Shape
{
}

final class Circle implements Shape
{
}

final class Unrelated
{
}

/** @param class-string<Shape> $name */
function takes_shape(string $name): void
{
}

/** @param class-string<Shape> $name */
function checks(Circle $circle, string $name): void
{
    /** @mago-expect analysis:impossible-condition,redundant-comparison */
    if (get_class($circle) === 'Unrelated') {
        echo 'never';
    }

    /** @mago-expect analysis:impossible-condition,redundant-comparison */
    if ($name === 'Unrelated') {
        echo 'never';
    }

    if ($name === 'Circle') {
        echo 'maybe';
    }

    if ($name === 'Not\Scanned\Class') {
        echo 'unknown classes are left alone';
    }

    /** @mago-expect analysis:invalid-argument */
    takes_shape('Unrelated');
    takes_shape('Circle');
}
