<?php

declare(strict_types=1);

/**
 * @template TData as array<string, mixed>
 */
abstract class Car
{
    /**
     * @return TData
     */
    abstract function getData(): array;
}

/**
 * @type DataArray = array{'Gewicht': int}
 *
 * @extends Car<self::DataArray>
 */
class RedCar extends Car
{
    public function getData(): array
    {
        return ['Gewicht' => 1000];
    }

    /**
     * @return DataArray
     **/
    public function getRaw(): array
    {
        return ['Gewicht' => 1000];
    }
}
