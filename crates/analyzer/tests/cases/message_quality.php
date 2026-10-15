<?php

declare(strict_types=1);

namespace App;

use Missing\External\UnknownBase;
use Missing\External\UnknownInterface;
use Missing\External\UnknownTrait;

// @mago-expect analysis:non-existent-class-like
class Child extends UnknownBase {}

// @mago-expect analysis:non-existent-class-like
class Implementation implements UnknownInterface {}

class TraitUser
{
    // @mago-expect analysis:non-existent-class-like
    use UnknownTrait;
}

function testNonObjectAccess(int $count): void
{
    // @mago-expect analysis:invalid-property-access
    $x = $count->items;

    // @mago-expect analysis:invalid-method-access
    $count->execute();
}
