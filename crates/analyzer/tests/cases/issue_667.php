<?php

declare(strict_types=1);

class GrandParentWithMethod
{
    /** @return object */
    public function getResponse(): object
    {
        return new stdClass();
    }
}

/** @method stdClass getResponse() */
class ParentWithDocblock extends GrandParentWithMethod
{
}

class ChildWithoutDocblock extends ParentWithDocblock
{
}

function test_inherited_pseudo_method(): stdClass
{
    $child = new ChildWithoutDocblock();
    $response = $child->getResponse();

    return $response;
}
