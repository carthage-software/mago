<?php

declare(strict_types=1);

function read_first_node_value(DOMNodeList $list): ?string
{
    return $list[0]->nodeValue;
}

function read_first_node(DOMNodeList $list): DOMNode|DOMNameSpaceNode|null
{
    return $list[0];
}

/**
 * @param DOMNodeList<DOMAttr> $attrs
 */
function read_first_attr_value(DOMNodeList $attrs): ?string
{
    return $attrs[0]->value;
}
