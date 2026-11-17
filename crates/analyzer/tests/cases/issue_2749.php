<?php

declare(strict_types=1);

final class Issue2749Item
{
    public function name(): string
    {
        return 'item';
    }
}

/**
 * @implements ArrayAccess<string, ?Issue2749Item>
 */
final class Issue2749Bag implements ArrayAccess
{
    public function offsetExists(mixed $offset): bool
    {
        return false;
    }

    public function offsetGet(mixed $offset): ?Issue2749Item
    {
        return null;
    }

    public function offsetSet(mixed $offset, mixed $value): void
    {
    }

    public function offsetUnset(mixed $offset): void
    {
    }
}

function issue_2749_attribute(SimpleXMLElement $node, string $name): ?string
{
    $attributes = $node->attributes();
    if (null === $attributes) {
        return null;
    }

    $value = $attributes[$name];

    return null === $value ? null : (string) $value;
}

function issue_2749_attribute_or(SimpleXMLElement $node, string $name): ?string
{
    $attributes = $node->attributes();
    if (null === $attributes || null === $attributes[$name]) {
        return null;
    }

    return $attributes[$name]->getName();
}

function issue_2749_take(SimpleXMLElement $element): string
{
    return $element->getName();
}

function issue_2749_unchecked(SimpleXMLElement $node): string
{
    /** @mago-expect analysis:possibly-null-argument */
    return issue_2749_take($node['x']);
}

function issue_2749_isset(SimpleXMLElement $node): string
{
    if (isset($node['x'])) {
        return $node['x']->getName();
    }

    return '';
}

function issue_2749_null_check(SimpleXMLElement $node): string
{
    if (null !== $node['x']) {
        return $node['x']->getName();
    }

    return '';
}

function issue_2749_bag_isset(Issue2749Bag $bag): string
{
    if (isset($bag['x'])) {
        return $bag['x']->name();
    }

    return '';
}

function issue_2749_bag_null_check(Issue2749Bag $bag): string
{
    if (null !== $bag['x']) {
        return $bag['x']->name();
    }

    return '';
}

/**
 * @param WeakMap<Issue2749Item, list<string>> $map
 *
 * @return list<string>
 */
function issue_2749_object_key(WeakMap $map, Issue2749Item $item): array
{
    if (!isset($map[$item])) {
        return [];
    }

    return $map[$item];
}
