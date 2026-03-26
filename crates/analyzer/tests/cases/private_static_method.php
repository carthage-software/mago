<?php

class X {
    private static function _x(): void
    {
    }
}

/** @mago-expect analysis:invalid-method-access */
X::_x();