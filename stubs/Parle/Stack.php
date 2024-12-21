<?php

namespace Parle;

use JetBrains\PhpStorm\Immutable;

/**
 * @template TValue
 */
class Stack
{
    /* Properties */
    /**
     * @var bool Whether the stack is empty, readonly.
     */
    #[Immutable]
    public $empty = true;

    /**
     * @var int Stack size, readonly.
     */
    #[Immutable]
    public $size = 0;

    /**
     * @var TValue Element on the top of the stack.
     */
    public $top;

    /* Methods */
    /**
     * Pop an item from the stack
     *
     * @link https://php.net/manual/en/parle-stack.pop.php
     * @return void
     */
    public function pop(): void {}

    /**
     * Push an item into the stack
     *
     * @link https://php.net/manual/en/parle-stack.push.php
     * @param TValue $item Variable to be pushed.
     * @return void
     */
    public function push($item) {}
}
