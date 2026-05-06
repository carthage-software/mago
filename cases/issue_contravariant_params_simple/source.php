<?php

declare(strict_types=1);

/**
 * @template Tk of array-key
 */
interface BaseInterface
{
    /**
     * @param Tk $k
     */
    public function get(int|string $k): mixed;
}

/**
 * @extends BaseInterface<int<0, max>>
 */
interface VectorInterface extends BaseInterface
{
    /**
     * @param int<0, max> $k
     */
    #[Override]
    public function get(int|string $k): mixed;
}

/**
 * @template Tk of array-key
 * @extends BaseInterface<Tk>
 */
interface MutableInterface extends BaseInterface {}

/**
 * @extends VectorInterface
 * @extends MutableInterface<int<0, max>>
 */
interface MutableVectorInterface extends MutableInterface, VectorInterface {}
