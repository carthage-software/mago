<?php

class BaseVehicle
{
}

class Car extends BaseVehicle
{
}

interface Drivable
{
}

interface Steerable extends Drivable
{
}

/**
 * @template T
 */
class GenericContainer
{
}

/**
 * @template TItem
 *
 * @extends GenericContainer<TItem>
 */
class Box extends GenericContainer
{
}

/**
 * @extends GenericContainer<string>
 */
class StringBox extends GenericContainer
{
}

/**
 * @template TKey
 * @template TValue
 */
class GenericPair
{
}

/**
 * @extends GenericPair<string, int>
 */
class StringIntPair extends GenericPair
{
}

/**
 * @template TFirst
 * @template TSecond
 *
 * @extends GenericPair<TSecond, TFirst>
 */
class FlippedPair extends GenericPair
{
}

/**
 * @template T
 */
interface Loader
{
}

/**
 * @template TItem
 *
 * @extends Loader<TItem>
 */
interface BulkLoader extends Loader
{
}

/**
 * @implements BulkLoader<Car>
 */
class Truck implements BulkLoader
{
}
