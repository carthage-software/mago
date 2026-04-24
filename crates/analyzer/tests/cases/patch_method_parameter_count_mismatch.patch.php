<?php

class VendorClass
{
    // @mago-expect analysis:patch-function-parameter-mismatch
    public function process(string $a, string $b, string $c): void {}
}
