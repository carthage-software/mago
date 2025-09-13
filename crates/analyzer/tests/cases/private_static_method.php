<?php

class X {
    private static function x(): void
    {
    }
}

/** @mago-expect analysis:invalid-method-access */
X::x();