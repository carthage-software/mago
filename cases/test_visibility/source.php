<?php

class P1
{
    public $prop;
}

class C1 extends P1
{
    private $prop; // Should error - narrowing visibility
}
