<?php

declare(strict_types=1);

function dom_child_nodes_are_nodes(DOMElement $element): string
{
    $text = '';
    foreach ($element->childNodes as $child) {
        $text .= $child->textContent;
        if ($child->childNodes->length > 0) {
            $text .= $child->nodeName;
        }
    }

    return $text;
}
