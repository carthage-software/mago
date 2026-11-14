<?php

declare(strict_types=1);

interface Item
{
    public function name(): string;
}

/**
 * @template T of Item|null
 */
final class Holder
{
    /** @var T */
    public mixed $item;

    /**
     * @param T $item
     */
    public function __construct(mixed $item)
    {
        $this->item = $item;
    }

    public function name(): string
    {
        if (isset($this->item)) {
            return $this->item->name();
        }

        return '';
    }

    public function early(): string
    {
        if (!isset($this->item)) {
            return '';
        }

        return $this->item->name();
    }
}
