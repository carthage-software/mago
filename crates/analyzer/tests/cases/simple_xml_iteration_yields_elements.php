<?php

declare(strict_types=1);

/**
 * @return list<string>
 */
function simple_xml_iteration_yields_elements(SimpleXMLElement $xml): array
{
    $names = [];
    foreach ($xml as $child) {
        $names[] = $child->getName();
    }

    return $names;
}
