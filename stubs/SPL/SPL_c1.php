<?php

class SplFileInfo implements Stringable
{
    public function __construct(string $filename) {}

    public function getPath(): string
    {
    }

    public function getFilename(): string
    {
    }

    public function getExtension(): string
    {
    }

    public function getBasename(string $suffix = ''): string
    {
    }

    public function getPathname(): string
    {
    }

    public function getPerms(): int|false
    {
    }

    /**
     * @throws RuntimeException
     */
    public function getInode(): int|false
    {
    }

    /**
     * @throws RuntimeException
     */
    public function getSize(): int|false
    {
    }

    /**
     * @throws RuntimeException
     */
    public function getOwner(): int|false
    {
    }

    /**
     * @throws RuntimeException
     */
    public function getGroup(): int|false
    {
    }

    /**
     * @throws RuntimeException
     */
    public function getATime(): int|false
    {
    }

    public function getMTime(): int|false
    {
    }

    /**
     * @throws RuntimeException
     */
    public function getCTime(): int|false
    {
    }

    /**
     * @throws RuntimeException
     */
    public function getType(): string|false
    {
    }

    public function isWritable(): bool
    {
    }

    public function isReadable(): bool
    {
    }

    public function isExecutable(): bool
    {
    }

    public function isFile(): bool
    {
    }

    public function isDir(): bool
    {
    }

    public function isLink(): bool
    {
    }

    /**
     * @throws RuntimeException
     */
    public function getLinkTarget(): string|false
    {
    }

    public function getRealPath(): string|false
    {
    }

    /**
     * @template T of SplFileInfo
     *
     * @param null|class-string<T> $class
     *
     * @return ($class is null ? SplFileInfo : T)
     */
    public function getFileInfo(null|string $class = null): SplFileInfo
    {
    }

    /**
     * @template T of SplFileInfo
     *
     * @param null|class-string<T> $class
     *
     * @return ($class is null ? null|SplFileInfo : null|T)
     */
    public function getPathInfo(null|string $class = null): null|SplFileInfo
    {
    }

    /**
     * @param null|resource $context
     *
     * @throws RuntimeException
     */
    public function openFile(string $mode = 'r', bool $useIncludePath = false, $context = null): SplFileObject
    {
    }

    /**
     * @template T of SplFileInfo
     *
     * @param class-string<T> $class
     */
    public function setFileClass(string $class = SplFileObject::class): void
    {
    }

    /**
     * @template T of SplFileInfo
     *
     * @param class-string<T> $class
     */
    public function setInfoClass(string $class = SplFileInfo::class): void
    {
    }

    /**
     * @return string
     */
    public function __toString(): string
    {
    }

    final public function _bad_state_ex(): void
    {
    }

    public function __wakeup()
    {
    }

    public function __debugInfo(): array
    {
    }
}

/**
 * @template-implements SeekableIterator<int, DirectoryIterator>
 */
class DirectoryIterator extends SplFileInfo implements SeekableIterator
{
    public function __construct(string $path) {}

    /**
     * @return null|DirectoryIterator
     *
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return null|int
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
     * @return void
     */
    public function rewind()
    {
    }

    /**
     * @param int $position
     */
    public function seek($position)
    {
    }

    /**
     * @return bool
     */
    public function valid()
    {
    }
}

/**
 * @implements Iterator<string, string|SplFileInfo>
 */
class FilesystemIterator extends DirectoryIterator implements Iterator
{
    const CURRENT_AS_PATHNAME = 32;
    const CURRENT_AS_FILEINFO = 0;
    const CURRENT_AS_SELF = 16;
    const CURRENT_MODE_MASK = 240;
    const KEY_AS_PATHNAME = 0;
    const KEY_AS_FILENAME = 256;
    const FOLLOW_SYMLINKS = 512;
    const KEY_MODE_MASK = 3840;
    const NEW_CURRENT_AND_KEY = 256;
    const SKIP_DOTS = 4096;
    const UNIX_PATHS = 8192;

    public function __construct(
        string $path,
        int $flags = self::KEY_AS_PATHNAME | self::CURRENT_AS_FILEINFO | self::SKIP_DOTS,
    ) {}

    /**
     * @return string|SplFileInfo|null
     *
     * @ignore-nullable-return
     */
    public function current(): string|SplFileInfo|null
    {
    }

    /**
     * @return string|null
     *
     * @ignore-nullable-return
     */
    public function key()
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
}

/**
 * @template-implements RecursiveIterator<string, RecursiveDirectoryIterator|string|SplFileInfo>
 * @template-implements SeekableIterator<string, RecursiveDirectoryIterator|string|SplFileInfo>
 */
class RecursiveDirectoryIterator extends FilesystemIterator implements RecursiveIterator, SeekableIterator
{
    const CURRENT_AS_PATHNAME = 32;
    const CURRENT_AS_FILEINFO = 0;
    const CURRENT_AS_SELF = 16;
    const CURRENT_MODE_MASK = 240;
    const KEY_AS_PATHNAME = 0;
    const KEY_AS_FILENAME = 256;
    const FOLLOW_SYMLINKS = 512;
    const KEY_MODE_MASK = 3840;
    const NEW_CURRENT_AND_KEY = 256;
    const SKIP_DOTS = 4096;
    const UNIX_PATHS = 8192;

    public function __construct(string $path, int $flags = self::KEY_AS_PATHNAME | self::CURRENT_AS_FILEINFO) {}

    public function getSubPath(): string
    {
    }

    public function getSubPathname(): string
    {
    }

    /**
     * @return RecursiveDirectoryIterator|string|SplFileInfo|null
     *
     * @ignore-nullable-return
     */
    public function current()
    {
    }

    /**
     * @return string|null
     *
     * @ignore-nullable-return
     */
    public function key()
    {
    }
}

/**
 * @template-extends FilesystemIterator<string, GlobIterator|SplFileInfo|string>
 */
class GlobIterator extends FilesystemIterator implements Countable
{
    /**
     * @return int<0, max>
     */
    public function count()
    {
    }
}

class SplFileObject extends SplFileInfo implements RecursiveIterator, SeekableIterator
{
    public const DROP_NEW_LINE = 1;
    public const READ_AHEAD = 2;
    public const SKIP_EMPTY = 4;
    public const READ_CSV = 8;

    /**
     * Construct a new file object.
     *
     * @link  https://php.net/manual/en/splfileobject.construct.php
     *
     * @param string $filename The file to open
     * @param string $mode [optional] The mode in which to open the file. See {@see fopen} for a list of allowed modes.
     * @param bool $useIncludePath [optional] Whether to search in the include_path for filename
     * @param resource $context [optional] A valid context resource created with {@see stream_context_create}
     *
     * @throws RuntimeException When the filename cannot be opened
     * @throws LogicException When the filename is a directory
     */
    public function __construct(string $filename, string $mode = 'r', bool $useIncludePath = false, $context = null) {}

    /**
     * Rewind the file to the first line
     * @link https://php.net/manual/en/splfileobject.rewind.php
     * @return void
     *
     * @throws RuntimeException If cannot be rewound
     */
    public function rewind(): void
    {
    }

    /**
     * Reached end of file
     * @link https://php.net/manual/en/splfileobject.eof.php
     * @return bool true if file is at EOF, false otherwise.
     */
    public function eof(): bool
    {
    }

    /**
     * Not at EOF
     * @link https://php.net/manual/en/splfileobject.valid.php
     * @return bool true if not reached EOF, false otherwise.
     */
    public function valid(): bool
    {
    }

    /**
     * @throws RuntimeException
     */
    public function fgets(): string
    {
    }

    public function fread(int $length): string|false
    {
    }

    /**
     * @return array|false|null
     */
    public function fgetcsv(string $separator = ',', string $enclosure = "\"", string $escape = "\\")
    {
    }

    public function fputcsv(
        array $fields,
        string $separator = ',',
        string $enclosure = '"',
        string $escape = "\\",
        string $eol = PHP_EOL,
    ): int|false {
    }

    public function setCsvControl(string $separator = ',', string $enclosure = "\"", string $escape = "\\"): void
    {
    }

    public function getCsvControl(): array
    {
    }

    public function flock(int $operation, int &$wouldBlock = null): bool
    {
    }

    public function fflush(): bool
    {
    }

    public function ftell(): int|false
    {
    }

    public function fseek(int $offset, int $whence = SEEK_SET): int
    {
    }

    public function fgetc(): string|false
    {
    }

    public function fpassthru(): int
    {
    }

    public function fscanf(string $format, mixed &...$vars): array|int|null
    {
    }

    public function fwrite(string $data, int $length = 0): int|false
    {
    }

    /**
     * Gets information about the file
     * @link https://php.net/manual/en/splfileobject.fstat.php
     * @return array an array with the statistics of the file; the format of the array
     * is described in detail on the <b>stat</b> manual page.
     */
    public function fstat(): array
    {
    }

    /**
     * Truncates the file to a given length
     * @link https://php.net/manual/en/splfileobject.ftruncate.php
     * @param int $size <p>
     * The size to truncate to.
     * </p>
     * <p>
     * If <i>size</i> is larger than the file it is extended with null bytes.
     * </p>
     * <p>
     * If <i>size</i> is smaller than the file, the extra data will be lost.
     * </p>
     * @return bool true on success or false on failure.
     */
    public function ftruncate(int $size): bool
    {
    }

    public function current(): string|array|false
    {
    }

    public function key(): int
    {
    }

    public function next(): void
    {
    }

    public function setFlags(int $flags): void
    {
    }

    public function getFlags(): int
    {
    }

    /**
     * @throws DomainException
     */
    public function setMaxLineLen(int $maxLength): void
    {
    }

    /**
     * @return int<0, max>
     */
    public function getMaxLineLen(): int
    {
    }

    /**
     * @return bool
     */
    public function hasChildren()
    {
    }

    /**
     * @return null|RecursiveIterator
     */
    public function getChildren()
    {
    }

    /**
     * @throws LogicException
     */
    public function seek(int $line): void
    {
    }

    public function getCurrentLine(): string
    {
    }

    public function __toString(): string
    {
    }
}

/**
 * The SplTempFileObject class offers an object oriented interface for a temporary file.
 * @link https://php.net/manual/en/class.spltempfileobject.php
 */
class SplTempFileObject extends SplFileObject
{
    /**
     * Construct a new temporary file object
     * @link https://php.net/manual/en/spltempfileobject.construct.php
     * @param int $maxMemory [optional]
     * @throws RuntimeException if an error occurs.
     * @since 5.1.2
     */
    public function __construct(int $maxMemory = 2097152) {}
}

/**
 * @template TValue
 * @template-implements Iterator<int, TValue>
 * @template-implements ArrayAccess<int, TValue>
 */
class SplDoublyLinkedList implements Iterator, Countable, ArrayAccess, Serializable
{
    public const IT_MODE_LIFO = 2;
    public const IT_MODE_FIFO = 0;
    public const IT_MODE_DELETE = 1;
    public const IT_MODE_KEEP = 0;

    /**
     * @param TValue $value
     */
    public function add(int $index, mixed $value): void
    {
    }

    /**
     * @return TValue
     */
    public function pop(): mixed
    {
    }

    /**
     * @return TValue
     */
    public function shift(): mixed
    {
    }

    /**
     * @param TValue $value
     */
    public function push(mixed $value): void
    {
    }

    /**
     * @param TValue $value
     */
    public function unshift(mixed $value): void
    {
    }

    /**
     * @return TValue
     */
    public function top(): mixed
    {
    }

    /**
     * @return TValue
     */
    public function bottom(): mixed
    {
    }

    public function count(): int
    {
    }

    public function isEmpty(): bool
    {
    }

    /**
     * <b>SplDoublyLinkedList::IT_MODE_LIFO</b> (Stack style)
     * @return int
     */
    public function setIteratorMode(int $mode): int
    {
    }

    /**
     * Returns the mode of iteration
     * @link https://php.net/manual/en/spldoublylinkedlist.getiteratormode.php
     * @return int the different modes and flags that affect the iteration.
     */
    public function getIteratorMode(): int
    {
    }

    /**
     * Returns whether the requested $index exists
     * @link https://php.net/manual/en/spldoublylinkedlist.offsetexists.php
     * @param mixed $index <p>
     * The index being checked.
     * </p>
     * @return bool true if the requested <i>index</i> exists, otherwise false
     */
    public function offsetExists($index): bool
    {
    }

    /**
     * Returns the value at the specified $index
     * @link https://php.net/manual/en/spldoublylinkedlist.offsetget.php
     * @param mixed $index <p>
     * The index with the value.
     * </p>
     * @return TValue The value at the specified <i>index</i>.
     */
    public function offsetGet($index): mixed
    {
    }

    /**
     * Sets the value at the specified $index to $newval
     * @link https://php.net/manual/en/spldoublylinkedlist.offsetset.php
     * @param mixed $index <p>
     * The index being set.
     * </p>
     * @param TValue $value <p>
     * The new value for the <i>index</i>.
     * </p>
     * @return void
     */
    public function offsetSet($index, mixed $value): void
    {
    }

    /**
     * Unsets the value at the specified $index
     * @link https://php.net/manual/en/spldoublylinkedlist.offsetunset.php
     * @param mixed $index <p>
     * The index being unset.
     * </p>
     * @return void
     */
    public function offsetUnset($index): void
    {
    }

    /**
     * Rewind iterator back to the start
     * @link https://php.net/manual/en/spldoublylinkedlist.rewind.php
     * @return void
     */
    public function rewind(): void
    {
    }

    /**
     * Return current array entry
     * @link https://php.net/manual/en/spldoublylinkedlist.current.php
     * @return TValue The current node value.
     */
    public function current(): mixed
    {
    }

    /**
     * Return current node index
     * @link https://php.net/manual/en/spldoublylinkedlist.key.php
     * @return string|float|int|bool|null The current node index.
     */
    public function key(): int
    {
    }

    /**
     * Move to next entry
     * @link https://php.net/manual/en/spldoublylinkedlist.next.php
     * @return void
     */
    public function next(): void
    {
    }

    /**
     * Move to previous entry
     * @link https://php.net/manual/en/spldoublylinkedlist.prev.php
     * @return void
     */
    public function prev(): void
    {
    }

    /**
     * Check whether the doubly linked list contains more nodes
     * @link https://php.net/manual/en/spldoublylinkedlist.valid.php
     * @return bool true if the doubly linked list contains any more nodes, false otherwise.
     */
    public function valid(): bool
    {
    }

    /**
     * Unserializes the storage
     * @link https://php.net/manual/en/spldoublylinkedlist.serialize.php
     * @param string $data The serialized string.
     * @return void
     * @since 5.4
     */
    public function unserialize(string $data): void
    {
    }

    /**
     * Serializes the storage
     * @link https://php.net/manual/en/spldoublylinkedlist.unserialize.php
     * @return string The serialized string.
     * @since 5.4
     */
    public function serialize(): string
    {
    }

    /**
     * @return array
     * @since 7.4
     */
    public function __debugInfo(): array
    {
    }

    /**
     * @return array
     * @since 7.4
     */
    public function __serialize(): array
    {
    }

    /**
     * @param array $data
     * @since 7.4
     */
    public function __unserialize(array $data): void
    {
    }
}

/**
 * @template TValue
 * The SplQueue class provides the main functionalities of a queue implemented using a doubly linked list.
 * @link https://php.net/manual/en/class.splqueue.php
 */
class SplQueue extends SplDoublyLinkedList
{
    /**
     * Adds an element to the queue.
     * @link https://php.net/manual/en/splqueue.enqueue.php
     * @param TValue $value <p>
     * The value to enqueue.
     * </p>
     * @return void
     */
    public function enqueue(mixed $value): void
    {
    }

    /**
     * Dequeues a node from the queue
     * @link https://php.net/manual/en/splqueue.dequeue.php
     * @return TValue The value of the dequeued node.
     */
    public function dequeue(): mixed
    {
    }

    /**
     * Sets the mode of iteration
     * @link https://php.net/manual/en/spldoublylinkedlist.setiteratormode.php
     * @param int $mode <p>
     * There are two orthogonal sets of modes that can be set:
     * </p>
     * The direction of the iteration (either one or the other):
     * <b>SplDoublyLinkedList::IT_MODE_LIFO</b> (Stack style)
     * @return void
     */
    public function setIteratorMode($mode)
    {
    }
}

/**
 * @template TValue
 * The SplStack class provides the main functionalities of a stack implemented using a doubly linked list.
 * @link https://php.net/manual/en/class.splstack.php
 * @template-extends SplDoublyLinkedList<TValue>
 */
class SplStack extends SplDoublyLinkedList
{
    /**
     * Sets the mode of iteration
     * @link https://php.net/manual/en/spldoublylinkedlist.setiteratormode.php
     * @param int $mode <p>
     * There are two orthogonal sets of modes that can be set:
     * </p>
     * The direction of the iteration (either one or the other):
     * <b>SplDoublyLinkedList::IT_MODE_LIFO</b> (Stack style)
     * @return void
     */
    public function setIteratorMode($mode)
    {
    }
}

/**
 * @template TValue
 * The SplHeap class provides the main functionalities of an Heap.
 * @link https://php.net/manual/en/class.splheap.php
 * @template-implements Iterator<int, TValue>
 */
abstract class SplHeap implements Iterator, Countable
{
    /**
     * Extracts a node from top of the heap and sift up.
     * @link https://php.net/manual/en/splheap.extract.php
     * @return TValue The value of the extracted node.
     */
    public function extract(): mixed
    {
    }

    /**
     * Inserts an element in the heap by sifting it up.
     * @link https://php.net/manual/en/splheap.insert.php
     * @param TValue $value <p>
     * The value to insert.
     * </p>
     * @return bool
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    public function insert(mixed $value)
    {
    }

    /**
     * Peeks at the node from the top of the heap
     * @link https://php.net/manual/en/splheap.top.php
     * @return TValue The value of the node on the top.
     */
    public function top(): mixed
    {
    }

    /**
     * Counts the number of elements in the heap.
     * @link https://php.net/manual/en/splheap.count.php
     * @return int the number of elements in the heap.
     */
    public function count(): int
    {
    }

    /**
     * Checks whether the heap is empty.
     * @link https://php.net/manual/en/splheap.isempty.php
     * @return bool whether the heap is empty.
     */
    public function isEmpty(): bool
    {
    }

    /**
     * Rewind iterator back to the start (no-op)
     * @link https://php.net/manual/en/splheap.rewind.php
     * @return void
     */
    public function rewind(): void
    {
    }

    /**
     * Return current node pointed by the iterator
     * @link https://php.net/manual/en/splheap.current.php
     * @return TValue The current node value.
     */
    public function current(): mixed
    {
    }

    /**
     * Return current node index
     * @link https://php.net/manual/en/splheap.key.php
     * @return int The current node index.
     */
    public function key(): int
    {
    }

    /**
     * Move to the next node
     * @link https://php.net/manual/en/splheap.next.php
     * @return void
     */
    public function next(): void
    {
    }

    /**
     * Check whether the heap contains more nodes
     * @link https://php.net/manual/en/splheap.valid.php
     * @return bool true if the heap contains any more nodes, false otherwise.
     */
    public function valid(): bool
    {
    }

    /**
     * Recover from the corrupted state and allow further actions on the heap.
     * @link https://php.net/manual/en/splheap.recoverfromcorruption.php
     * @return bool
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    public function recoverFromCorruption()
    {
    }

    /**
     * Compare elements in order to place them correctly in the heap while sifting up.
     * @link https://php.net/manual/en/splheap.compare.php
     * @param mixed $value1 <p>
     * The value of the first node being compared.
     * </p>
     * @param mixed $value2 <p>
     * The value of the second node being compared.
     * </p>
     * @return int Result of the comparison, positive integer if <i>value1</i> is greater than <i>value2</i>, 0 if they are equal, negative integer otherwise.
     * </p>
     * <p>
     * Having multiple elements with the same value in a Heap is not recommended. They will end up in an arbitrary relative position.
     */
    abstract protected function compare($value1, $value2);

    /**
     * @return bool
     */
    public function isCorrupted(): bool
    {
    }

    /**
     * @return array
     * @since 7.4
     */
    public function __debugInfo(): array
    {
    }
}

/**
 * @template TValue
 * The SplMinHeap class provides the main functionalities of a heap, keeping the minimum on the top.
 * @link https://php.net/manual/en/class.splminheap.php
 * @template-extends SplHeap<TValue>
 */
class SplMinHeap extends SplHeap
{
    /**
     * Compare elements in order to place them correctly in the heap while sifting up.
     * @link https://php.net/manual/en/splminheap.compare.php
     * @param TValue $value1 <p>
     * The value of the first node being compared.
     * </p>
     * @param TValue $value2 <p>
     * The value of the second node being compared.
     * </p>
     * @return int Result of the comparison, positive integer if <i>value1</i> is lower than <i>value2</i>, 0 if they are equal, negative integer otherwise.
     * </p>
     * <p>
     * Having multiple elements with the same value in a Heap is not recommended. They will end up in an arbitrary relative position.
     */
    protected function compare(mixed $value1, mixed $value2): int
    {
    }

    /**
     * Extracts a node from top of the heap and sift up.
     * @link https://php.net/manual/en/splheap.extract.php
     * @return TValue The value of the extracted node.
     */
    public function extract()
    {
    }

    /**
     * Inserts an element in the heap by sifting it up.
     * @link https://php.net/manual/en/splheap.insert.php
     * @param TValue $value <p>
     * The value to insert.
     * </p>
     * @return true
     */
    public function insert($value)
    {
    }

    /**
     * Peeks at the node from the top of the heap
     * @link https://php.net/manual/en/splheap.top.php
     * @return TValue The value of the node on the top.
     */
    public function top()
    {
    }

    /**
     * Counts the number of elements in the heap.
     * @link https://php.net/manual/en/splheap.count.php
     * @return int the number of elements in the heap.
     */
    public function count()
    {
    }

    /**
     * Checks whether the heap is empty.
     * @link https://php.net/manual/en/splheap.isempty.php
     * @return bool whether the heap is empty.
     */
    public function isEmpty()
    {
    }

    /**
     * Rewind iterator back to the start (no-op)
     * @link https://php.net/manual/en/splheap.rewind.php
     * @return void
     */
    public function rewind()
    {
    }

    /**
     * Return current node pointed by the iterator
     * @link https://php.net/manual/en/splheap.current.php
     * @return TValue The current node value.
     */
    public function current()
    {
    }

    /**
     * Return current node index
     * @link https://php.net/manual/en/splheap.key.php
     * @return int The current node index.
     */
    public function key()
    {
    }

    /**
     * Move to the next node
     * @link https://php.net/manual/en/splheap.next.php
     * @return void
     */
    public function next()
    {
    }

    /**
     * Check whether the heap contains more nodes
     * @link https://php.net/manual/en/splheap.valid.php
     * @return bool true if the heap contains any more nodes, false otherwise.
     */
    public function valid()
    {
    }

    /**
     * Recover from the corrupted state and allow further actions on the heap.
     * @link https://php.net/manual/en/splheap.recoverfromcorruption.php
     * @return void
     */
    public function recoverFromCorruption()
    {
    }
}

/**
 * @template TValue
 * The SplMaxHeap class provides the main functionalities of a heap, keeping the maximum on the top.
 * @link https://php.net/manual/en/class.splmaxheap.php
 * @template-extends SplHeap<TValue>
 */
class SplMaxHeap extends SplHeap
{
    /**
     * Compare elements in order to place them correctly in the heap while sifting up.
     * @link https://php.net/manual/en/splmaxheap.compare.php
     * @param TValue $value1 <p>
     * The value of the first node being compared.
     * </p>
     * @param TValue $value2 <p>
     * The value of the second node being compared.
     * </p>
     * @return int Result of the comparison, positive integer if <i>value1</i> is greater than <i>value2</i>, 0 if they are equal, negative integer otherwise.
     * </p>
     * <p>
     * Having multiple elements with the same value in a Heap is not recommended. They will end up in an arbitrary relative position.
     */
    protected function compare(mixed $value1, mixed $value2): int
    {
    }
}

/**
 * @template TPriority
 * @template TValue
 * The SplPriorityQueue class provides the main functionalities of an
 * prioritized queue, implemented using a heap.
 * @link https://php.net/manual/en/class.splpriorityqueue.php
 * @template-implements Iterator<int, TValue>
 */
class SplPriorityQueue implements Iterator, Countable
{
    public const EXTR_BOTH = 3;
    public const EXTR_PRIORITY = 2;
    public const EXTR_DATA = 1;

    /**
     * Compare priorities in order to place elements correctly in the heap while sifting up.
     * @link https://php.net/manual/en/splpriorityqueue.compare.php
     * @param TPriority $priority1 <p>
     * The priority of the first node being compared.
     * </p>
     * @param TPriority $priority2 <p>
     * The priority of the second node being compared.
     * </p>
     * @return int Result of the comparison, positive integer if <i>priority1</i> is greater than <i>priority2</i>, 0 if they are equal, negative integer otherwise.
     * </p>
     * <p>
     * Multiple elements with the same priority will get dequeued in no particular order.
     */
    public function compare(mixed $priority1, mixed $priority2): int
    {
    }

    /**
     * Inserts an element in the queue by sifting it up.
     * @link https://php.net/manual/en/splpriorityqueue.insert.php
     * @param TValue $value <p>
     * The value to insert.
     * </p>
     * @param TPriority $priority <p>
     * The associated priority.
     * </p>
     * @return true
     */
    public function insert(mixed $value, mixed $priority): true
    {
    }

    /**
     * Sets the mode of extraction
     * @link https://php.net/manual/en/splpriorityqueue.setextractflags.php
     * @param int $flags <p>
     * Defines what is extracted by <b>SplPriorityQueue::current</b>,
     * <b>SplPriorityQueue::top</b> and
     * <b>SplPriorityQueue::extract</b>.
     * </p>
     * <b>SplPriorityQueue::EXTR_DATA</b> (0x00000001): Extract the data
     * @return int
     */
    public function setExtractFlags(int $flags): int
    {
    }

    /**
     * Peeks at the node from the top of the queue
     * @link https://php.net/manual/en/splpriorityqueue.top.php
     * @return TValue The value or priority (or both) of the top node, depending on the extract flag.
     */
    public function top(): mixed
    {
    }

    /**
     * Extracts a node from top of the heap and sift up.
     * @link https://php.net/manual/en/splpriorityqueue.extract.php
     * @return TValue The value or priority (or both) of the extracted node, depending on the extract flag.
     */
    public function extract(): mixed
    {
    }

    /**
     * Counts the number of elements in the queue.
     * @link https://php.net/manual/en/splpriorityqueue.count.php
     * @return int the number of elements in the queue.
     */
    public function count(): int
    {
    }

    /**
     * Checks whether the queue is empty.
     * @link https://php.net/manual/en/splpriorityqueue.isempty.php
     * @return bool whether the queue is empty.
     */
    public function isEmpty(): bool
    {
    }

    /**
     * Rewind iterator back to the start (no-op)
     * @link https://php.net/manual/en/splpriorityqueue.rewind.php
     * @return void
     */
    public function rewind(): void
    {
    }

    /**
     * Return current node pointed by the iterator
     * @link https://php.net/manual/en/splpriorityqueue.current.php
     * @return TValue The value or priority (or both) of the current node, depending on the extract flag.
     */
    public function current(): mixed
    {
    }

    /**
     * Return current node index
     * @link https://php.net/manual/en/splpriorityqueue.key.php
     * @return int The current node index.
     */
    public function key(): int
    {
    }

    /**
     * Move to the next node
     * @link https://php.net/manual/en/splpriorityqueue.next.php
     * @return void
     */
    public function next(): void
    {
    }

    /**
     * Check whether the queue contains more nodes
     * @link https://php.net/manual/en/splpriorityqueue.valid.php
     * @return bool true if the queue contains any more nodes, false otherwise.
     */
    public function valid(): bool
    {
    }

    /**
     * Recover from the corrupted state and allow further actions on the queue.
     * @link https://php.net/manual/en/splpriorityqueue.recoverfromcorruption.php
     */
    public function recoverFromCorruption(): true
    {
    }

    /**
     * @return bool
     */
    public function isCorrupted(): bool
    {
    }

    /**
     * @return int
     */
    public function getExtractFlags(): int
    {
    }

    /**
     * @return array
     * @since 7.4
     */
    public function __debugInfo(): array
    {
    }
}

/**
 * @template TValue
 *
 * @template-implements Iterator<int, TValue>
 * @template-implements ArrayAccess<int, TValue>
 * @template-implements IteratorAggregate<int, TValue>
 */
class SplFixedArray implements Iterator, ArrayAccess, Countable, IteratorAggregate, JsonSerializable
{
    public function __construct(int $size = 0) {}

    /**
     * @return int<0, max>
     */
    public function count(): int
    {
    }

    /**
     * @return list<TValue>
     */
    public function toArray(): array
    {
    }

    /**
     * Import a PHP array in a <b>SplFixedArray</b> instance
     * @link https://php.net/manual/en/splfixedarray.fromarray.php
     * @param array $array <p>
     * The array to import.
     * </p>
     * @param bool $preserveKeys [optional] <p>
     * Try to save the numeric indexes used in the original array.
     * </p>
     * @return SplFixedArray an instance of <b>SplFixedArray</b>
     * containing the array content.
     */
    public static function fromArray(array $array, bool $preserveKeys = true): SplFixedArray
    {
    }

    /**
     * @return int<0, max>
     */
    public function getSize(): int
    {
    }

    /**
     * @return bool
     */
    public function setSize(int $size)
    {
    }

    /**
     * @param int $index
     */
    public function offsetExists($index): bool
    {
    }

    /**
     * @param int $index
     *
     * @return TValue
     */
    public function offsetGet($index): mixed
    {
    }

    /**
     * @param int $index
     * @param TValue $value
     */
    public function offsetSet($index, mixed $value): void
    {
    }

    /**
     * @param int $index
     */
    public function offsetUnset($index): void
    {
    }

    /**
     * @return void
     */
    public function rewind()
    {
    }

    /**
     * @return TValue
     */
    public function current()
    {
    }

    /**
     * @return int
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
    public function valid(): bool
    {
    }

    #[Deprecated('The function is deprecated', since: '8.4')]
    public function __wakeup(): void
    {
    }

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }

    /**
     * @return Iterator<int, TValue>
     */
    public function getIterator(): Iterator
    {
    }

    public function jsonSerialize(): array
    {
    }
}

/**
 * The <b>SplObserver</b> interface is used alongside
 * <b>SplSubject</b> to implement the Observer Design Pattern.
 * @link https://php.net/manual/en/class.splobserver.php
 */
interface SplObserver
{
    /**
     * Receive update from subject
     * @link https://php.net/manual/en/splobserver.update.php
     * @param SplSubject $subject <p>
     * The <b>SplSubject</b> notifying the observer of an update.
     * </p>
     * @return void
     */
    public function update(SplSubject $subject): void;
}

/**
 * The <b>SplSubject</b> interface is used alongside
 * <b>SplObserver</b> to implement the Observer Design Pattern.
 * @link https://php.net/manual/en/class.splsubject.php
 */
interface SplSubject
{
    /**
     * Attach an SplObserver
     * @link https://php.net/manual/en/splsubject.attach.php
     * @param SplObserver $observer <p>
     * The <b>SplObserver</b> to attach.
     * </p>
     * @return void
     */
    public function attach(SplObserver $observer): void;

    /**
     * Detach an observer
     * @link https://php.net/manual/en/splsubject.detach.php
     * @param SplObserver $observer <p>
     * The <b>SplObserver</b> to detach.
     * </p>
     * @return void
     */
    public function detach(SplObserver $observer): void;

    /**
     * Notify an observer
     * @link https://php.net/manual/en/splsubject.notify.php
     * @return void
     */
    public function notify(): void;
}

/**
 * @template TObject of object
 * @template TValue
 * The SplObjectStorage class provides a map from objects to data or, by
 * ignoring data, an object set. This dual purpose can be useful in many
 * cases involving the need to uniquely identify objects.
 * @link https://php.net/manual/en/class.splobjectstorage.php
 * @template-implements Iterator<int, TObject>
 * @template-implements ArrayAccess<TObject, TValue>
 */
class SplObjectStorage implements Countable, SeekableIterator, Serializable, ArrayAccess
{
    /**
     * Adds an object in the storage
     * @link https://php.net/manual/en/splobjectstorage.attach.php
     * @param TObject $object <p>
     * The object to add.
     * </p>
     * @param TValue $info [optional] <p>
     * The data to associate with the object.
     * </p>
     * @return void
     */
    public function attach(object $object, mixed $info = null): void
    {
    }

    /**
     * Removes an object from the storage
     * @link https://php.net/manual/en/splobjectstorage.detach.php
     * @param TObject $object <p>
     * The object to remove.
     * </p>
     * @return void
     */
    public function detach(object $object): void
    {
    }

    /**
     * Checks if the storage contains a specific object
     * @link https://php.net/manual/en/splobjectstorage.contains.php
     * @param TObject $object <p>
     * The object to look for.
     * </p>
     * @return bool true if the object is in the storage, false otherwise.
     */
    public function contains(object $object): bool
    {
    }

    /**
     * Adds all objects from another storage
     * @link https://php.net/manual/en/splobjectstorage.addall.php
     * @param SplObjectStorage<TObject, TValue> $storage <p>
     * The storage you want to import.
     * </p>
     * @return int
     */
    public function addAll(#[LanguageLevelTypeAware(['8.0' => 'SplObjectStorage'], default: '')]  $storage): int
    {
    }

    /**
     * Removes objects contained in another storage from the current storage
     * @link https://php.net/manual/en/splobjectstorage.removeall.php
     * @param SplObjectStorage<TObject, TValue> $storage <p>
     * The storage containing the elements to remove.
     * </p>
     * @return int
     */
    public function removeAll(#[LanguageLevelTypeAware(['8.0' => 'SplObjectStorage'], default: '')]  $storage): int
    {
    }

    /**
     * Removes all objects except for those contained in another storage from the current storage
     * @link https://php.net/manual/en/splobjectstorage.removeallexcept.php
     * @param SplObjectStorage<TObject, TValue> $storage <p>
     * The storage containing the elements to retain in the current storage.
     * </p>
     * @return int
     * @since 5.3.6
     */
    public function removeAllExcept(
        #[LanguageLevelTypeAware(['8.0' => 'SplObjectStorage'], default: '')]  $storage,
    ): int {
    }

    /**
     * Returns the data associated with the current iterator entry
     * @link https://php.net/manual/en/splobjectstorage.getinfo.php
     * @return TValue The data associated with the current iterator position.
     */
    public function getInfo(): mixed
    {
    }

    /**
     * Sets the data associated with the current iterator entry
     * @link https://php.net/manual/en/splobjectstorage.setinfo.php
     * @param TValue $info <p>
     * The data to associate with the current iterator entry.
     * </p>
     * @return void
     */
    public function setInfo(mixed $info): void
    {
    }

    /**
     * Returns the number of objects in the storage
     * @link https://php.net/manual/en/splobjectstorage.count.php
     * @param int $mode [optional]
     * @return int The number of objects in the storage.
     */
    public function count(#[PhpStormStubsElementAvailable(from: '8.0')] int $mode = COUNT_NORMAL): int
    {
    }

    /**
     * Rewind the iterator to the first storage element
     * @link https://php.net/manual/en/splobjectstorage.rewind.php
     * @return void
     */
    public function rewind(): void
    {
    }

    /**
     * Returns if the current iterator entry is valid
     * @link https://php.net/manual/en/splobjectstorage.valid.php
     * @return bool true if the iterator entry is valid, false otherwise.
     */
    public function valid(): bool
    {
    }

    /**
     * Returns the index at which the iterator currently is
     * @link https://php.net/manual/en/splobjectstorage.key.php
     * @return int The index corresponding to the position of the iterator.
     */
    public function key(): int
    {
    }

    /**
     * Returns the current storage entry
     * @link https://php.net/manual/en/splobjectstorage.current.php
     * @return TObject The object at the current iterator position.
     */
    public function current(): object
    {
    }

    /**
     * Move to the next entry
     * @link https://php.net/manual/en/splobjectstorage.next.php
     * @return void
     */
    public function next(): void
    {
    }

    /**
     * Unserializes a storage from its string representation
     * @link https://php.net/manual/en/splobjectstorage.unserialize.php
     * @param string $data <p>
     * The serialized representation of a storage.
     * </p>
     * @return void
     * @since 5.2.2
     */
    public function unserialize(string $data): void
    {
    }

    /**
     * Serializes the storage
     * @link https://php.net/manual/en/splobjectstorage.serialize.php
     * @return string A string representing the storage.
     * @since 5.2.2
     */
    public function serialize(): string
    {
    }

    /**
     * Checks whether an object exists in the storage
     * @link https://php.net/manual/en/splobjectstorage.offsetexists.php
     * @param TObject $object <p>
     * The object to look for.
     * </p>
     * @return bool true if the object exists in the storage,
     * and false otherwise.
     */
    public function offsetExists($object): bool
    {
    }

    /**
     * Associates data to an object in the storage
     * @link https://php.net/manual/en/splobjectstorage.offsetset.php
     * @param TObject $object <p>
     * The object to associate data with.
     * </p>
     * @param TValue $info [optional] <p>
     * The data to associate with the object.
     * </p>
     * @return void
     */
    public function offsetSet(
        #[LanguageLevelTypeAware(['8.1' => 'mixed'], default: '')]  $object,
        mixed $info = null,
    ): void {
    }

    /**
     * Removes an object from the storage
     * @link https://php.net/manual/en/splobjectstorage.offsetunset.php
     * @param TObject $object <p>
     * The object to remove.
     * </p>
     * @return void
     */
    public function offsetUnset($object): void
    {
    }

    /**
     * Returns the data associated with an <type>object</type>
     * @link https://php.net/manual/en/splobjectstorage.offsetget.php
     * @param TObject $object <p>
     * The object to look for.
     * </p>
     * @return TValue The data previously associated with the object in the storage.
     */
    public function offsetGet($object): mixed
    {
    }

    /**
     * Calculate a unique identifier for the contained objects
     * @link https://php.net/manual/en/splobjectstorage.gethash.php
     * @param TObject $object <p>
     * object whose identifier is to be calculated.
     * </p>
     * @return string A string with the calculated identifier.
     * @since 5.4
     */
    public function getHash(object $object): string
    {
    }

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }

    public function __debugInfo(): array
    {
    }

    public function seek(int $offset): void
    {
    }
}

class MultipleIterator implements Iterator
{
    public const MIT_NEED_ANY = 0;
    public const MIT_NEED_ALL = 1;
    public const MIT_KEYS_NUMERIC = 0;
    public const MIT_KEYS_ASSOC = 2;

    public function __construct(int $flags = MultipleIterator::MIT_NEED_ALL | MultipleIterator::MIT_KEYS_NUMERIC) {}

    public function getFlags(): int
    {
    }

    public function setFlags(int $flags): void
    {
    }

    public function attachIterator(Iterator $iterator, int|string|null $info = null): void
    {
    }

    public function detachIterator(Iterator $iterator): void
    {
    }

    public function containsIterator(Iterator $iterator): bool
    {
    }

    public function countIterators(): int
    {
    }

    public function rewind(): void
    {
    }

    public function valid(): bool
    {
    }

    public function key(): array
    {
    }

    /**
     * @throws RuntimeException
     * @throws InvalidArgumentException
     */
    public function current(): array
    {
    }

    public function next(): void
    {
    }

    public function __debugInfo(): array
    {
    }
}
