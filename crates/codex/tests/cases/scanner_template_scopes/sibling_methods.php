<?php

namespace App;

use Lib\Item;

class Container
{
    /**
     * @template Item
     * @return Item
     */
    public function generic() {}

    /** @return Item */
    public function sibling() {}
}

/** @return Item */
function after_class() {}
