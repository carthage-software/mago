<?php

class P1
{
    public $prop;
}

class C1 extends P1
{
    // @mago-expect analysis:incompatible-property-access
    // @mago-expect analysis:unused-property
    private $prop;  // Should error - narrowing visibility
}
