<?php

/**
 * @return list<class-string>
 */
function spl_classes(): array
{
}

/**
 * @param class-string $class
 */
function spl_autoload(string $class, null|string $file_extensions = null): void
{
}

function spl_autoload_extensions(null|string $file_extensions = null): string
{
}

/**
 * Register given function as __autoload() implementation
 * @link https://php.net/manual/en/function.spl-autoload-register.php
 * @param callable|null $callback [optional] <p>
 * The autoload function being registered.
 * If no parameter is provided, then the default implementation of
 * spl_autoload will be registered.
 * </p>
 * @param bool $throw This parameter specifies whether spl_autoload_register() should throw exceptions when the
 * autoload_function cannot be registered. Ignored since since 8.0.
 * @param bool $prepend If true, spl_autoload_register() will prepend the autoloader on the autoload stack instead of
 * appending it.
 * @return bool true on success or false on failure.
 * @throws TypeError Since 8.0.
 * @since 5.1.2
 */
function spl_autoload_register(null|callable $callback, bool $throw = true, bool $prepend = false): bool
{
}

/**
 * Unregister given function as __autoload() implementation
 * @link https://php.net/manual/en/function.spl-autoload-unregister.php
 * @param callable $callback <p>
 * The autoload function being unregistered.
 * </p>
 * @return bool true on success or false on failure.
 * @since 5.1.2
 */
function spl_autoload_unregister(callable $callback): bool
{
}

/**
 * Return all registered __autoload() functions
 * @link https://php.net/manual/en/function.spl-autoload-functions.php
 * @return array|false An array of all registered __autoload functions.
 * If the autoload stack is not activated then the return value is false.
 * If no function is registered the return value will be an empty array.
 * @since 5.1.2
 */
function spl_autoload_functions(): array
{
}

/**
 * @param class-string $class
 */
function spl_autoload_call(string $class): void
{
}

/**
 * Return the parent classes of the given class
 * @link https://php.net/manual/en/function.class-parents.php
 * @param object|string $object_or_class <p>
 * An object (class instance) or a string (class name).
 * </p>
 * @param bool $autoload [optional] <p>
 * Whether to allow this function to load the class automatically through
 * the __autoload magic
 * method.
 * </p>
 * @return string[]|false An array on success, or false on error.
 *
 * @pure
 */
function class_parents($object_or_class, bool $autoload = true): array|false
{
}

/**
 * Return the interfaces which are implemented by the given class
 * @link https://php.net/manual/en/function.class-implements.php
 * @param object|string $object_or_class <p>
 * An object (class instance) or a string (class name).
 * </p>
 * @param bool $autoload [optional] <p>
 * Whether to allow this function to load the class automatically through
 * the __autoload magic
 * method.
 * </p>
 * @return string[]|false An array on success, or false on error.
 */
#[Pure]
function class_implements($object_or_class, bool $autoload = true): array|false
{
}

/**
 * Return hash id for given object
 * @link https://php.net/manual/en/function.spl-object-hash.php
 * @param object $object
 * @return string A string that is unique for each object and is always the same for
 * the same object.
 */
#[Pure]
function spl_object_hash(object $object): string
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param Traversable<K, V>|array<K, V> $iterator <p>
 *
 * @return ($preserve_keys is true ? array<K, V> : list<V>)
 */
function iterator_to_array(Traversable|array $iterator, bool $preserve_keys = true): array
{
}

/**
 * @return int<0, max>
 */
function iterator_count(Traversable|array $iterator): int
{
}

/**
 * Call a function for every element in an iterator
 * @link https://php.net/manual/en/function.iterator-apply.php
 * @param Traversable $iterator <p>
 * The class to iterate over.
 * </p>
 * @param callable $callback <p>
 * The callback function to call on every element.
 * The function must return true in order to
 * continue iterating over the iterator.
 * </p>
 * @param array|null $args
 *
 * @return int the iteration count.
 */
function iterator_apply(Traversable $iterator, callable $callback, null|array $args = null): int
{
}

/**
 * @param object|class-string $object_or_class
 *
 * @return list<trait-string>|false
 *
 * @pure
 */
function class_uses($object_or_class, bool $autoload = true): array|false
{
}

function spl_object_id(object $object): int
{
}
