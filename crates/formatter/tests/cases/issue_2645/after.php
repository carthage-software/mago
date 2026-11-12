<?php

class A
{
    /********************* processing ****************d*g**/

    public function normalize(): void {}

    /*** three ***/
    public function three(): void {}

    /****/
    public $four = 1;

    /******/
    public $six = 1;

    /**** @var int */
    public $decorated = 1;

    public function trailing(): void {} /*** trailing ***/

    /*******************
     * multi-line banner
     *******************/
    public function multi(): void {}

    /** @var int */
    public $tagged = 1;

    /** text */
    public $text = 1;

    /** @var int */
    public $spaced = 1;
}

/********************* functions *********************/

function first(): void {}
