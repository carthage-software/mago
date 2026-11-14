<?php

declare(strict_types=1);

function count_links(DOMDocument $document): int
{
    $xpath = new DOMXPath($document);

    $nodes = $xpath->query('//a');
    if ($nodes === false) {
        return 0;
    }

    return $nodes->count();
}

function sixth(DOMDocument $document): ?DOMElement
{
    $list = $document->getElementsByTagName('div');

    return $list[5];
}

function sixth_strict(DOMDocument $document): DOMElement
{
    $list = $document->getElementsByTagName('div');

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $list[5];
}
