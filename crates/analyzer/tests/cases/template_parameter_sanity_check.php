<?php

/**
 * @template Tk of array-key
 * @template Tv
 */
interface CollectionInterface
{
}

/**
 * @template Tk of array-key
 * @template Tv
 */
interface IndexAccessInterface
{
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends CollectionInterface<Tk, Tv>
 * @extends IndexAccessInterface<Tk, Tv>
 */
interface AccessibleCollectionInterface extends CollectionInterface, IndexAccessInterface
{
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends IndexAccessInterface<Tk, Tv>
 */
interface MutableIndexAccessInterface extends IndexAccessInterface
{
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends CollectionInterface<Tk, Tv>
 */
interface MutableCollectionInterface extends CollectionInterface
{
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends AccessibleCollectionInterface<Tk, Tv>
 * @extends MutableCollectionInterface<Tk, Tv>
 * @extends MutableIndexAccessInterface<Tk, Tv>
 */
interface MutableAccessibleCollectionInterface extends
    AccessibleCollectionInterface,
    MutableCollectionInterface,
    MutableIndexAccessInterface
{
}

/**
 * @template T of array-key
 *
 * @extends AccessibleCollectionInterface<T, T>
 */
interface SetInterface extends AccessibleCollectionInterface
{
}

/**
 * @template T of array-key
 *
 * @extends SetInterface<T>
 * @extends MutableAccessibleCollectionInterface<T, T>
 */
interface MutableSetInterface extends MutableAccessibleCollectionInterface, SetInterface
{
}
