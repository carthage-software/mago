<?php

class LogicException extends Exception
{
}

class BadFunctionCallException extends LogicException
{
}

class BadMethodCallException extends BadFunctionCallException
{
}

class DomainException extends LogicException
{
}

class InvalidArgumentException extends LogicException
{
}

class LengthException extends LogicException
{
}

class OutOfRangeException extends LogicException
{
}

class RuntimeException extends Exception
{
}

class OutOfBoundsException extends RuntimeException
{
}

class OverflowException extends RuntimeException
{
}

class RangeException extends RuntimeException
{
}

class UnderflowException extends RuntimeException
{
}

class UnexpectedValueException extends RuntimeException
{
}

/**
 * @template-implements Iterator<never, never>
 */
class EmptyIterator implements Iterator
{
    /**
     * @return never
     */
    public function current()
    {
    }

    /**
     * @return never
     */
    public function key()
    {
    }

    /**
     * @return void
     */
    public function next()
    {
    }

    /**
     * @return void
     */
    public function rewind()
    {
    }

    /**
     * @return false
     */
    public function valid()
    {
    }
}

/**
 * @template-covariant K
 * @template-covariant V
 * @template-covariant TIterator as Iterator<K, V>
 *
 * @template-implements OuterIterator<K, V>
 *
 * @template-extends FilterIterator<K, V, TIterator>
 */
class CallbackFilterIterator extends FilterIterator implements OuterIterator
{
    /**
     * @param TIterator $iterator
     * @param (callable(V, K, TIterator): bool) $callback
     */
    public function __construct(Iterator $iterator, callable $callback) {}

    /**
     * @return V|null
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template K
 * @template V
 * @template TIterator as RecursiveIterator<K, V>
 *
 * @template-implements RecursiveIterator<K, V>
 * @template-extends CallbackFilterIterator<K, V, TIterator>
 */
class RecursiveCallbackFilterIterator extends CallbackFilterIterator implements RecursiveIterator
{
    /**
     * @param TIterator $iterator
     * @param (callable(V, K, TIterator): bool) $callback
     */
    public function __construct(RecursiveIterator $iterator, callable $callback) {}

    /**
     * @return RecursiveCallbackFilterIterator<K, V, TIterator>
     */
    public function getChildren()
    {
    }

    /**
     * @return bool
     */
    public function hasChildren()
    {
    }

    /**
     * @return V|null
     */
    public function current()
    {
    }

    /**
     * @return K|null
     */
    public function key()
    {
    }
}

/**
 * @template-covariant K
 * @template-covariant V
 *
 * @template-extends Iterator<K, V>
 */
interface RecursiveIterator extends Iterator
{
    /**
     * @return bool
     */
    public function hasChildren();

    /**
     * @return RecursiveIterator<K, V>
     */
    public function getChildren();
}

/**
 * @template K
 * @template-covariant V
 *
 * @implements OuterIterator<K, V>
 */
class RecursiveIteratorIterator implements OuterIterator
{
    public const LEAVES_ONLY = 0;
    public const SELF_FIRST = 1;
    public const CHILD_FIRST = 2;
    public const CATCH_GET_CHILD = 16;

    /**
     * @param Traversable<K, V> $iterator
     */
    public function __construct(Traversable $iterator, int $mode = self::LEAVES_ONLY, int $flags = 0) {}

    public function rewind(): void
    {
    }

    public function valid(): bool
    {
    }

    /**
     * @return null|K
     */
    public function key(): mixed
    {
    }

    /**
     * @return null|V
     */
    public function current(): mixed
    {
    }

    public function next(): void
    {
    }

    public function getDepth(): int
    {
    }

    /**
     * @return RecursiveIterator<K, V>|null
     */
    public function getSubIterator(null|int $level): null|RecursiveIterator
    {
    }

    /**
     * @return RecursiveIterator<K, V>
     */
    public function getInnerIterator(): RecursiveIterator
    {
    }

    public function beginIteration(): void
    {
    }

    public function endIteration(): void
    {
    }

    public function callHasChildren(): bool
    {
    }

    /**
     * @return RecursiveIterator<K, V>|null
     */
    public function callGetChildren(): null|RecursiveIterator
    {
    }

    public function beginChildren(): void
    {
    }

    public function endChildren(): void
    {
    }

    public function nextElement(): void
    {
    }

    public function setMaxDepth(int $maxDepth = -1): void
    {
    }

    public function getMaxDepth(): int|false
    {
    }
}

/**
 * @template-covariant K
 * @template-covariant V
 *
 * @template-extends Iterator<K, V>
 */
interface OuterIterator extends Iterator
{
    /**
     * @return Iterator<K, V>
     */
    public function getInnerIterator();
}

/**
 * @template K
 * @template-covariant V
 *
 * @implements OuterIterator<K, V>
 */
class IteratorIterator implements OuterIterator
{
    /**
     * @param Traversable<K, V> $iterator
     * @param class-string|null $class
     */
    public function __construct(Traversable $iterator, null|string $class = null) {}

    /**
     * @return Iterator<K, V>|null
     */
    public function getInnerIterator(): null|Iterator
    {
    }

    /**
     * @return void
     */
    public function rewind(): void
    {
    }

    public function valid(): bool
    {
    }

    /**
     * @return null|K
     */
    public function key(): mixed
    {
    }

    /**
     * @return null|V
     */
    public function current(): mixed
    {
    }

    public function next(): void
    {
    }
}

/**
 * @template-covariant K
 * @template-covariant V
 * @template-covariant TIterator as Traversable<K, V>
 *
 * @template-extends IteratorIterator<K, V, TIterator>
 */
abstract class FilterIterator extends IteratorIterator
{
    /** @return bool */
    abstract public function accept();

    /**
     * @return null|V
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return null|K
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template K
 * @template V
 * @template TIterator as RecursiveIterator<K, V>
 *
 * @template-extends FilterIterator<K, V, TIterator>
 * @template-implements RecursiveIterator<K, V>
 */
abstract class RecursiveFilterIterator extends FilterIterator implements RecursiveIterator
{
    /**
     * @param TIterator $iterator
     */
    public function __construct(RecursiveIterator $iterator) {}

    /**
     * @return TIterator
     */
    public function getChildren()
    {
    }

    /**
     * @return bool
     */
    public function hasChildren()
    {
    }

    /**
     * @return V|null
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template K
 * @template V
 * @template TIterator as RecursiveIterator<K, V>
 *
 * @template-extends RecursiveFilterIterator<K, V, TIterator>
 */
class ParentIterator extends RecursiveFilterIterator implements RecursiveIterator, OuterIterator
{
    /**
     * @return bool
     */
    public function accept()
    {
    }

    /**
     * @param TIterator $iterator
     */
    public function __construct(RecursiveIterator $iterator) {}

    /**
     * @return ParentIterator<K,V>
     */
    public function getChildren()
    {
    }

    /**
     * @return bool
     */
    public function hasChildren()
    {
    }

    /**
     * @return void
     */
    public function next()
    {
    }

    /**
     * @return void
     */
    public function rewind()
    {
    }

    /**
     * @return V|null
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template-covariant K
 * @template-covariant V
 *
 * @template-extends Iterator<K, V>
 */
interface SeekableIterator extends Iterator
{
    public function seek(int $position): void;
}

/**
 * @template-covariant K
 * @template-covariant V
 * @template-covariant TIterator as Iterator<K, V>
 *
 * @template-implements OuterIterator<K, V>
 * @template-extends IteratorIterator<K, V, TIterator>
 */
class LimitIterator extends IteratorIterator implements OuterIterator
{
    /**
     * @param TIterator $iterator
     */
    public function __construct(Iterator $iterator, int $offset = 0, int $limit = -1) {}

    /**
     * @return V|null
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template-covariant K
 * @template-covariant V
 * @template-covariant TIterator as Iterator<K, V>
 *
 * @template-implements OuterIterator<K, V>
 * @template-implements ArrayAccess<K, V>
 *
 * @template-extends IteratorIterator<K, V, TIterator>
 */
class CachingIterator extends IteratorIterator implements OuterIterator, ArrayAccess, Countable
{
    const CALL_TOSTRING = 1;
    const CATCH_GET_CHILD = 16;
    const TOSTRING_USE_KEY = 2;
    const TOSTRING_USE_CURRENT = 4;
    const TOSTRING_USE_INNER = 8;
    const FULL_CACHE = 256;

    /**
     * @param TIterator $iterator
     */
    public function __construct(Iterator $iterator, int $flags = self::CALL_TOSTRING) {}

    /**
     * @return bool
     */
    public function hasNext()
    {
    }

    /**
     * @return V|null
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     * @ignore-nullable-return
     */
    public function key()
    {
    }

    /**
     * @return array<array-key, V>
     */
    public function getCache()
    {
    }
}

/**
 * @template K
 * @template V
 * @template TIterator as Iterator<K, V>
 *
 * @template-implements RecursiveIterator<K, V>
 * @template-extends CachingIterator<K, V, TIterator>
 */
class RecursiveCachingIterator extends CachingIterator implements RecursiveIterator
{
    /**
     * @return RecursiveCachingIterator<K,V, TIterator>
     */
    public function getChildren()
    {
    }

    /**
     * @return bool
     */
    public function hasChildren()
    {
    }
}

/**
 * @template-covariant K
 * @template-covariant V
 * @template-covariant TIterator as Iterator<K, V>
 *
 * @template-extends IteratorIterator<K, V, TIterator>
 */
class NoRewindIterator extends IteratorIterator
{
    /**
     * @param TIterator $iterator
     */
    public function __construct(Iterator $iterator) {}

    /**
     * @return V|null
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template K
 * @template V
 * @template TIterator as Iterator<K, V>
 *
 * @template-extends IteratorIterator<K, V, TIterator>
 */
class AppendIterator extends IteratorIterator
{
    public function __construct() {}

    /**
     * @param TIterator $iterator
     * @return void
     */
    public function append(Iterator $iterator)
    {
    }

    /**
     * @return ArrayIterator<K, V>
     */
    public function getArrayIterator()
    {
    }

    /**
     * @return int
     */
    public function getIteratorIndex()
    {
    }

    /**
     * @return V|null
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template-covariant K
 * @template-covariant V
 * @template-covariant TIterator as Iterator<K, V>
 *
 * @template-extends IteratorIterator<K, V, TIterator>
 */
class InfiniteIterator extends IteratorIterator
{
    /**
     * @param TIterator $iterator
     */
    public function __construct(Iterator $iterator) {}

    /**
     * @return V|null
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template K
 * @template V
 * @template TIterator as Iterator<K, V>
 *
 * @template-extends FilterIterator<K, V, TIterator>
 */
class RegexIterator extends FilterIterator
{
    const MATCH = 0;
    const GET_MATCH = 1;
    const ALL_MATCHES = 2;
    const SPLIT = 3;
    const REPLACE = 4;
    const USE_KEY = 1;

    /**
     * @param TIterator $iterator
     * @param string $regex
     * @param RegexIterator::MATCH|RegexIterator::GET_MATCH|RegexIterator::ALL_MATCHES|RegexIterator::SPLIT|RegexIterator::REPLACE $mode
     * @param 0|RegexIterator::USE_KEY $flags
     */
    public function __construct(
        Iterator $iterator,
        string $regex,
        int $mode = self::MATCH,
        int $flags = 0,
        int $preg_flags = 0,
    ) {}

    /**
     * @return null|V
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return null|K
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template K
 * @template V
 * @template TIterator as RecursiveIterator<K, V>
 *
 * @template-implements RecursiveIterator<K, V>
 * @template-extends RegexIterator<K, V, TIterator>
 */
class RecursiveRegexIterator extends RegexIterator implements RecursiveIterator
{
    const MATCH = 0;
    const GET_MATCH = 1;
    const ALL_MATCHES = 2;
    const SPLIT = 3;
    const REPLACE = 4;
    const USE_KEY = 1;

    /**
     * @param TIterator $iterator
     * @param string $regex
     * @param RecursiveRegexIterator::MATH|RecursiveRegexIterator::GET_MATCH|RecursiveRegexIterator::ALL_MATCHES|RecursiveRegexIterator::SPLIT|RecursiveRegexIterator::REPLACE $mode
     * @param RecursiveRegexIterator::USE_KEY|0 $flags
     * @param int $preg_flags
     */
    public function __construct(
        RecursiveIterator $iterator,
        string $regex,
        int $mode = self::MATCH,
        int $flags = 0,
        int $preg_flags = 0,
    ) {}

    /**
     * @return RecursiveRegexIterator<K, V>
     */
    public function getChildren()
    {
    }
}

/**
 * @template K
 * @template V
 *
 * @template-extends RecursiveIteratorIterator<K, V>
 * @template-implements OuterIterator<K, V>
 */
class RecursiveTreeIterator extends RecursiveIteratorIterator implements OuterIterator
{
    const LEAVES_ONLY = 0;
    const SELF_FIRST = 1;
    const CHILD_FIRST = 2;
    const CATCH_GET_CHILD = 16;

    const BYPASS_CURRENT = 4;
    const BYPASS_KEY = 8;
    const PREFIX_LEFT = 0;
    const PREFIX_MID_HAS_NEXT = 1;
    const PREFIX_MID_LAST = 2;
    const PREFIX_END_HAS_NEXT = 3;
    const PREFIX_END_LAST = 4;
    const PREFIX_RIGHT = 5;

    /**
     * @return void
     */
    public function beginChildren()
    {
    }

    /**
     * @return RecursiveIterator
     */
    public function beginIteration()
    {
    }

    /**
     * @return RecursiveIterator
     */
    public function callGetChildren()
    {
    }

    /**
     * @return bool
     */
    public function callHasChildren()
    {
    }

    /**
     * @param RecursiveIterator<K, V>|IteratorAggregate<K, V> $it
     * @param int $flags
     * @param RecursiveTreeIterator::CATCH_GET_CHILD $cit_flags
     * @param RecursiveTreeIterator::LEAVES_ONLY|RecursiveTreeIterator::SELF_FIRST|RecursiveTreeIterator::CHILD_FIRST $mode
     */
    public function __construct(
        $it,
        int $flags = self::BYPASS_KEY,
        int $cit_flags = self::CATCH_GET_CHILD,
        int $mode = self::SELF_FIRST,
    ) {}

    /**
     * @return null|V
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    public function endChildren(): void
    {
    }

    public function endIteration(): void
    {
    }

    public function getEntry(): string
    {
    }

    public function getPostfix(): string
    {
    }

    public function getPrefix(): string
    {
    }

    /**
     * @return null|K
     * @ignore-nullable-return
     */
    public function key()
    {
    }

    public function next(): void
    {
    }

    public function nextElement(): void
    {
    }

    public function rewind(): void
    {
    }

    public function setPostfix(string $postfix): void
    {
    }

    public function setPrefixPart(int $part, string $value): void
    {
    }

    public function valid(): bool
    {
    }
}

/**
 * @template K of array-key
 * @template V
 *
 * @template-implements IteratorAggregate<K, V>
 * @template-implements ArrayAccess<K, V>
 */
class ArrayObject implements IteratorAggregate, ArrayAccess, Serializable, Countable
{
    const STD_PROP_LIST = 1;
    const ARRAY_AS_PROPS = 2;

    /**
     * @param array<K, V>|object $input
     * @param int $flags
     * @param class-string<ArrayIterator<K,V>>|class-string<ArrayObject<K,V>> $iterator_class
     */
    public function __construct($input = null, $flags = 0, $iterator_class = 'ArrayIterator') {}

    /**
     * @param K $offset
     *
     * @return bool
     *
     * @no-named-arguments
     */
    public function offsetExists($offset)
    {
    }

    /**
     * @param K $offset
     *
     * @return V
     *
     * @no-named-arguments
     */
    public function offsetGet($offset)
    {
    }

    /**
     * @param K $offset
     * @param V $value
     *
     * @return void
     *
     * @no-named-arguments
     */
    public function offsetSet($offset, $value)
    {
    }

    /**
     * @param K $offset
     *
     * @return void
     *
     * @no-named-arguments
     */
    public function offsetUnset($offset)
    {
    }

    /**
     * @param V $value
     *
     * @return void
     */
    public function append($value)
    {
    }

    /**
     * @return array<K, V>
     */
    public function getArrayCopy()
    {
    }

    /**
     * @return int
     */
    public function count()
    {
    }

    /**
     * @return int
     */
    public function getFlags()
    {
    }

    /**
     * @param int $flags
     *
     * @return void
     */
    public function setFlags($flags)
    {
    }

    /**
     * @return void
     */
    public function asort()
    {
    }

    /**
     * @return void
     */
    public function ksort()
    {
    }

    /**
     * @param (callable(V, V): int) $cmp_function
     *
     * @return void
     */
    public function uasort($cmp_function)
    {
    }

    /**
     * @param (callable(K, K):int) $cmp_function
     *
     * @return void
     */
    public function uksort($cmp_function)
    {
    }

    /**
     * @return void
     */
    public function natsort()
    {
    }

    /**
     * @return void
     */
    public function natcasesort()
    {
    }

    /**
     * @param string $serialized
     *
     * @return void
     */
    public function unserialize($serialized)
    {
    }

    /**
     * @return string
     */
    public function serialize()
    {
    }

    /**
     * @return ArrayIterator<K, V>
     */
    public function getIterator()
    {
    }

    /**
     * @param mixed $input
     *
     * @return array
     */
    public function exchangeArray($input)
    {
    }

    /**
     * @param class-string<ArrayIterator<K,V>>|class-string<ArrayObject<K,V>> $iterator_class
     *
     * @return void
     */
    public function setIteratorClass($iterator_class)
    {
    }

    /**
     * @return class-string<ArrayIterator<K, V>>|class-string<ArrayObject<K, V>>
     */
    public function getIteratorClass()
    {
    }
}

/**
 * @template K as array-key
 * @template V
 *
 * @template-implements SeekableIterator<K, V>
 * @template-implements ArrayAccess<K, V>
 */
class ArrayIterator implements SeekableIterator, ArrayAccess, Serializable, Countable
{
    const STD_PROP_LIST = 1;
    const ARRAY_AS_PROPS = 2;

    /**
     * @param array<K, V> $array
     * @param int $flags
     */
    public function __construct($array = [], $flags = 0) {}

    /**
     * @param K $offset
     *
     * @return bool
     */
    public function offsetExists($offset)
    {
    }

    /**
     * @param K $offset
     *
     * @return V|null
     *
     * @ignore-nullable-return
     */
    public function offsetGet($offset)
    {
    }

    /**
     * @param K $offset
     *
     * @param V $value
     *
     * @return void
     */
    public function offsetSet($offset, $value)
    {
    }

    /**
     * @param K $offset
     *
     * @return void
     */
    public function offsetUnset($offset)
    {
    }

    /**
     * @param V $value
     *
     * @return void
     */
    public function append($value)
    {
    }

    /**
     * @return array<K, V>
     */
    public function getArrayCopy()
    {
    }

    /**
     * @return int
     */
    public function count()
    {
    }

    /**
     * @return int
     */
    public function getFlags()
    {
    }

    /**
     * @param int $flags
     *
     * @return void
     */
    public function setFlags($flags)
    {
    }

    /**
     * @return void
     */
    public function asort()
    {
    }

    /**
     * @return void
     */
    public function ksort()
    {
    }

    /**
     * @param (callable(V,V): int) $cmp_function
     *
     * @return void
     */
    public function uasort($cmp_function)
    {
    }

    /**
     * @param (callable(K,K): int) $cmp_function
     *
     * @return void
     */
    public function uksort($cmp_function)
    {
    }

    /**
     * @return void
     */
    public function natsort()
    {
    }

    /**
     * @return void
     */
    public function natcasesort()
    {
    }

    /**
     * @param string $serialized
     *
     * @return void
     */
    public function unserialize($serialized)
    {
    }

    /**
     * @return string
     */
    public function serialize()
    {
    }

    /**
     * @return void
     */
    public function rewind()
    {
    }

    /**
     * @return V|null
     *
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     *
     * @ignore-nullable-return
     */
    public function key()
    {
    }

    /**
     * @return void
     */
    public function next()
    {
    }

    /**
     * @return bool
     */
    public function valid()
    {
    }

    /**
     * @param int $position
     *
     * @return void
     */
    public function seek($position)
    {
    }
}

/**
 * @template K
 * @template V
 *
 * @template-implements RecursiveIterator<K, V>
 * @template-extends ArrayIterator<K, V>
 */
class RecursiveArrayIterator extends ArrayIterator implements RecursiveIterator
{
    const STD_PROP_LIST = 1;
    const ARRAY_AS_PROPS = 2;
    const CHILD_ARRAYS_ONLY = 4;

    /**
     * @return ?RecursiveArrayIterator<K, V>
     */
    public function getChildren()
    {
    }

    /**
     * @return bool
     */
    public function hasChildren()
    {
    }

    /**
     * @return V|null
     *
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return K|null
     *
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}
