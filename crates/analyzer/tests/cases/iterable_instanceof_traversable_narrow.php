<?php

declare(strict_types=1);

interface Subscriber {}

final class Factory
{
    /**
     * @return iterable<Subscriber>
     */
    public function create(): iterable
    {
        yield new class implements Subscriber {};
    }
}

/**
 * @return array<array-key, Subscriber>
 */
function via_ternary(Factory $factory): array
{
    $subscribers = $factory->create();

    return $subscribers instanceof Traversable
        ? iterator_to_array($subscribers, false)
        : $subscribers;
}

/**
 * @return array<array-key, Subscriber>
 */
function via_if(Factory $factory): array
{
    $subscribers = $factory->create();

    if ($subscribers instanceof Traversable) {
        $subscribers = iterator_to_array($subscribers, false);
    }

    return $subscribers;
}
