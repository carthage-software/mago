<?php

declare(strict_types=1);

class Fixture
{
}

/**
 * @template T of object
 */
interface Parser
{
    /**
     * @param T $object
     * @return array<string, T>
     */
    public function toArray(object $object): array;
}

/**
 * @implements Parser<Fixture>
 */
class FixtureParser implements Parser
{
    public function toArray(object $object): array
    {
        return [
            'fixture' => $object,
        ];
    }
}

/**
 * @implements Parser<Fixture>
 */
class OtherFixtureParser implements Parser
{
    /**
     * @inheritDoc
     */
    public function toArray(object $object): array
    {
        return [
            'fixture' => $object,
        ];
    }
}
