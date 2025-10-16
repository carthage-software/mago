<?php

declare(strict_types=1);

class Status
{
    public function __construct(
        public string $value,
    ) {}
}

class Filter
{
    public function accept(Status $status): bool
    {
        return true;
    }
}

/**
 * @param list<string> $items
 * @return Generator<Status>
 */
function processItems(array $items, Filter $filter): Generator
{
    foreach ($items as $item) {
        if ($filter->accept($status = new Status($item))) {
            yield $status;
        }
    }
}

/**
 * @param list<string> $items
 * @return Generator<Status>
 */
function processItemsWithAnd(array $items, Filter $filter): Generator
{
    foreach ($items as $item) {
        if ($item !== '' && $filter->accept($status = new Status($item))) {
            yield $status;
        }
    }
}

/**
 * @param list<string> $items
 * @return Generator<Status>
 */
function processItemsWithOr(array $items, Filter $filter): Generator
{
    foreach ($items as $item) {
        if ($item !== '' || $filter->accept($status = new Status($item))) {
            /**
             * @mago-expect analysis:undefined-variable
             * @mago-expect analysis:invalid-yield-value-type
             */
            yield $status;
        }
    }
}
