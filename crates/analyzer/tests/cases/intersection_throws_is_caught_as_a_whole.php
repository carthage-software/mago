<?php

declare(strict_types=1);

interface IntersectionThrowsMarker
{
}

final class IntersectionThrowsException extends RuntimeException implements IntersectionThrowsMarker
{
}

/**
 * @throws IntersectionThrowsMarker&\Throwable
 */
function intersection_throws_run(): void
{
    throw new IntersectionThrowsException();
}

function intersection_throws_caught_as_throwable(): void
{
    try {
        intersection_throws_run();
    } catch (\Throwable) {
    }
}

function intersection_throws_caught_as_marker(): void
{
    try {
        intersection_throws_run();
    } catch (IntersectionThrowsMarker) {
    }
}

/**
 * @throws \Throwable
 */
function intersection_throws_declared_as_throwable(): void
{
    intersection_throws_run();
}
