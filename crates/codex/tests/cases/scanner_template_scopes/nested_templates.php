<?php

namespace App;

/** @template T of object */
class Container
{
    /**
     * @template U of T
     * @return U
     */
    public function generic()
    {
        /** @return T */
        function nested() {}
    }

    /** @return T */
    public function sibling() {}
}
