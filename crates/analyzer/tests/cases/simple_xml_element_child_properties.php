<?php

declare(strict_types=1);

final class SimpleXmlChildElement extends SimpleXMLElement
{
}

function simple_xml_child(SimpleXMLElement $node): SimpleXMLElement
{
    return $node->child;
}

function simple_xml_nested(SimpleXMLElement $node): string
{
    return (string) $node->channel->item->title;
}

function simple_xml_dashed(SimpleXMLElement $node): SimpleXMLElement
{
    return $node->{'some-tag'};
}

function simple_xml_subclass_child(SimpleXmlChildElement $node): SimpleXmlChildElement
{
    return $node->child;
}

function simple_xml_write(SimpleXMLElement $node, SimpleXMLElement $other): void
{
    $node->text = 'value';
    $node->number = 1;
    $node->empty = null;
    $node->copy = $other;
}

function simple_xml_write_array(SimpleXMLElement $node): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    $node->items = [1];
}

function simple_xml_isset_unset(SimpleXMLElement $node): bool
{
    unset($node->child);

    return isset($node->other);
}

function simple_xml_isset_chain(SimpleXMLElement $node): bool
{
    return isset($node->response) && isset($node->response->note->condition);
}

function simple_xml_coalesce(SimpleXMLElement $node, SimpleXMLElement $default): SimpleXMLElement
{
    return $node->child ?? $default;
}

final class SimpleXmlMagicIsset
{
    public function __get(string $name): string
    {
        return $name;
    }

    public function __isset(string $name): bool
    {
        return false;
    }
}

function magic_isset_coalesce(SimpleXmlMagicIsset $object): string
{
    /** @mago-expect analysis:non-documented-property */
    return $object->name ?? 'none';
}
