<?php

declare(strict_types=1);

class Player1375 {}
class Team1375 {}

/**
 * @template T of object
 * @param class-string<T> $className
 * @return list<T>
 */
function executeSearch1375(string $className): array { return []; }

/**
 * @template T of Player1375|Team1375
 * @param class-string<T> $className
 * @return list<T>
 */
function advancedSearch1375(string $className): array
{
    $_ = match ($className) {
        Player1375::class => 'players',
        Team1375::class   => 'teams',
        default       => throw new \LogicException(),
    };

    return executeSearch1375($className);
}
