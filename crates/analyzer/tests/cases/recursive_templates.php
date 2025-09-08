<?php

/**
 * @template Tk of array-key
 * @template Tv
 */
final class Map
{
    /**
     * @var array<Tk, Tv> $elements
     */
    private array $elements;

    /**
     * @param array<Tk, Tv> $elements
     */
    public function __construct(array $elements = [])
    {
        $this->elements = $elements;
    }

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return Map<Tk, array{0: Tv, 1: Tu}>
     */
    public function zip(array $elements): Map
    {
        return $this->zip($elements);
    }

    /**
     * @return list<Tk>
     */
    public function keys(): array
    {
        return array_keys($this->elements);
    }

    /**
     * @return list<Tv>
     */
    public function values(): array
    {
        return array_values($this->elements);
    }

    /**
     * @return array{
     *   keys: list<Tk>,
     *   values: list<Tv>,
     *   zipped_with_keys: Map<Tk, array{0: Tv, 1: Tk}>,
     *   zipped_with_values: Map<Tk, array{0: Tv, 1: Tv}>,
     *   values_of_zipped_with_keys: list<array{0: Tv, 1: Tk}>,
     *   values_of_zipped_with_values: list<array{0: Tv, 1: Tv}>
     * }
     */
    public function test(): array
    {
        $keys = $this->keys();
        $values = $this->values();
        $zipped_with_keys = $this->zip($keys);
        $zipped_with_values = $this->zip($values);
        $values_of_zipped_with_keys = $zipped_with_keys->values();
        $values_of_zipped_with_values = $zipped_with_values->values();

        return [
            'keys' => $keys,
            'values' => $values,
            'zipped_with_keys' => $zipped_with_keys,
            'zipped_with_values' => $zipped_with_values,
            'values_of_zipped_with_keys' => $values_of_zipped_with_keys,
            'values_of_zipped_with_values' => $values_of_zipped_with_values,
        ];
    }
}
