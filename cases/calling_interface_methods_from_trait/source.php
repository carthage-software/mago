<?php

declare(strict_types=1);

interface FactoryInterface
{
    public static function createInstance(): static;

    /**
     * Creates multiple instances of the implementing class.
     *
     * @param int<1, max> $count The number of instances to create.
     *
     * @return non-empty-list<static> An array containing the created instances.
     */
    public static function createMultipleInstances(int $count): array;

    public function doSomething(): static;

    public function doSomethingTwice(): static;
}

/**
 * @require-implements FactoryInterface
 */
trait FactoryConvenienceMethodsTrait
{
    /**
     * Creates multiple instances of the implementing class.
     *
     * @param int<1, max> $count The number of instances to create.
     *
     * @return non-empty-list<static> An array containing the created instances.
     */
    public static function createMultipleInstances(int $count): array
    {
        echo "Creating {$count} instances.. kinda";

        return [
            static::createInstance(),
        ];
    }

    public function doSomethingTwice(): static
    {
        $this->doSomething();

        return $this->doSomething();
    }
}

/**
 * @consistent-constructor
 */
final class Storage implements FactoryInterface
{
    use FactoryConvenienceMethodsTrait;

    public static function createInstance(): static
    {
        return new static();
    }

    public function doSomething(): static
    {
        echo 'test';

        return $this;
    }
}
