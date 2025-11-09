<?php declare(strict_types=1);

// Test @inheritDoc with nested generic types

/**
 * @template TKey
 * @template TValue
 */
interface Collection
{
    /**
     * @param array<TKey, TValue> $items Items to add
     * @return void
     */
    public function addAll(array $items): void;

    /**
     * @return array<TKey, array<int, TValue>> Grouped items
     */
    public function groupBy(): array;

    /**
     * @param TKey $key
     * @return TValue|null
     */
    public function get(mixed $key): mixed;
}

/**
 * @implements Collection<string, int>
 */
class IntCollection implements Collection
{
    /** @inheritDoc */
    public function addAll(array $items): void
    {
        foreach ($items as $key => $value) {
            echo $key . ': ' . $value;
        }
    }

    /** @inheritDoc */
    public function groupBy(): array
    {
        return ['group1' => [1, 2, 3]];
    }

    /** @inheritDoc */
    public function get(mixed $key): mixed
    {
        return 42;
    }
}

$collection = new IntCollection();
$collection->addAll(['a' => 1, 'b' => 2]);
$grouped = $collection->groupBy();
$value = $collection->get('test');

echo 'Grouped: ';
foreach ($grouped as $group => $items) {
    echo $group . ' => [' . implode(', ', $items) . '] ';
}

echo "Value: $value\n";
