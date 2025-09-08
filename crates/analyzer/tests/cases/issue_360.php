<?php

/**
 * @return list<int>
 */
function foo(): array
{
    $a = null;
    $a[] = 1;
    return $a;
}

class Item
{
}

class Data
{
    /** @var array<int, Item> */
    private array $items;

    public function __construct(
        private DateTimeImmutable $fromDate,
        private DateTimeImmutable $toDate,
    ) {
        $this->items = [];
    }

    /**
     * @return array{
     *  'from_date': DateTimeImmutable,
     *  'to_date': DateTimeImmutable,
     *  'items'?: list<Item>
     * }
     */
    public function toArray(): array
    {
        $params = [
            'from_date' => $this->fromDate,
            'to_date' => $this->toDate,
        ];

        foreach ($this->items as $item) {
            $items = $params['items']; // @mago-expect analysis:possibly-undefined-string-array-index
            $items[] = $item;
            $params['items'] = $items;
        }

        return $params;
    }

    /**
     * @return array{
     *  'from_date': DateTimeImmutable,
     *  'to_date': DateTimeImmutable,
     *  'items'?: non-empty-list<Item>
     * }
     */
    public function toArrayImplicit(): array
    {
        $params = [
            'from_date' => $this->fromDate,
            'to_date' => $this->toDate,
        ];

        foreach ($this->items as $item) {
            $params['items'][] = $item;
        }

        return $params;
    }

    /**
     * @param array{
     *  'from_date': DateTimeImmutable,
     *  'to_date': DateTimeImmutable,
     * } $params
     *
     * @return array{
     *  'from_date': DateTimeImmutable,
     *  'to_date': DateTimeImmutable,
     *  'items'?: non-empty-list<Item>
     * }
     */
    public function addItemstoParams(array $params): array
    {
        foreach ($this->items as $item) {
            $params['items'][] = $item;
        }

        return $params;
    }

    /**
     * @param array{
     *  'from_date': DateTimeImmutable,
     *  'to_date': DateTimeImmutable,
     *  ...
     * } $params
     *
     * @return array{
     *  'from_date': DateTimeImmutable,
     *  'to_date': DateTimeImmutable,
     *  'items'?: non-empty-list<Item>,
     *  ...
     * }
     */
    public function addItemstoParamsWithExtra(array $params): array
    {
        foreach ($this->items as $item) {
            // @mago-expect analysis:undefined-string-array-index
            // @mago-expect analysis:mixed-array-assignment
            $params['items'][] = $item;
        }

        return $params;
    }
}
