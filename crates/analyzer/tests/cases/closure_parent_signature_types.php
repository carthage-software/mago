<?php

class Base {}

class Child extends Base
{
    public function native(parent $value): parent
    {
        $copy = static function (parent $item): parent {
            return $item;
        };

        return $value;
    }

    /**
     * @param parent $value
     * @return parent
     */
    public function documented($value)
    {
        $copy = /**
         * @param parent $item
         * @return parent
         */ static fn($item) => $item;

        return $value;
    }
}
