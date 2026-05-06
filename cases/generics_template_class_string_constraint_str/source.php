<?php

declare(strict_types=1);

interface GenAnimalCs {}

final class GenDogCs implements GenAnimalCs {}

final class GenCarCs {}

/**
 * @template T of GenAnimalCs
 */
final class GenZooCs
{
    /** @param class-string<T> $c */
    public function __construct(
        public string $c,
    ) {}
}

new GenZooCs(GenCarCs::class);
