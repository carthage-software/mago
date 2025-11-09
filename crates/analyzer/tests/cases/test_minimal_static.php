<?php

class P1
{
    public $prop;
}

class C1 extends P1
{
    // @mago-expect analysis:incompatible-static-modifier
    public static $prop;
}
