<?php

declare(strict_types=1);

function testSimpleXmlTruthy(SimpleXMLElement $xml): bool
{
    if ($xml) {
        return true;
    }

    return false;
}

function testSimpleXmlFalsy(SimpleXMLElement $xml): bool
{
    if (!$xml) {
        return false;
    }

    return true;
}

function testSimpleXmlTernary(SimpleXMLElement $xml): string
{
    return $xml ? 'has content' : 'empty';
}

function testSimpleXmlIterator(SimpleXMLIterator $xml): bool
{
    if ($xml) {
        return true;
    }

    return false;
}
