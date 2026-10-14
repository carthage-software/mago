<?php

declare(strict_types=1);

interface Marker
{
}

final class Broken extends \RuntimeException implements Marker
{
}

interface Validator
{
    /**
     * @mago-expect analysis:throws-type-not-throwable
     *
     * @param array<string, mixed> $event
     * @throws Marker
     */
    public function validate(array $event): void;
}

final class RequiredFieldValidator implements Validator
{
    public function validate(array $event): void
    {
        if (!isset($event['id'])) {
            throw new Broken('id missing');
        }
    }
}

/**
 * @mago-expect analysis:throws-type-not-throwable
 * @throws int
 */
function scalar_throws(): void
{
}

/**
 * @throws \RuntimeException
 */
function valid_throws(): void
{
}

/**
 * @throws UndefinedProblem
 */
function unknown_throws_is_skipped(): void
{
}

/**
 * @throws mixed
 */
function mixed_throws_is_skipped(): void
{
}
