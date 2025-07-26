<?php

class SimpleXMLElement implements Traversable, ArrayAccess, Countable, Iterator, Stringable, RecursiveIterator
{
    /**
     * @throws Exception
     */
    public function __construct(
        string $data,
        int $options = 0,
        bool $dataIsURL = false,
        string $namespaceOrPrefix = '',
        bool $isPrefix = false,
    ) {}

    /**
     * @param string $name child name
     *
     * @return static
     */
    private function __get($name)
    {
    }

    public function asXML(string|null $filename = null): string|bool
    {
    }

    public function saveXML(string|null $filename = null): string|bool
    {
    }

    /**
     * @return list<SimpleXMLElement>|false|null
     */
    public function xpath(string $expression): array|false|null
    {
    }

    public function registerXPathNamespace(string $prefix, string $namespace): bool
    {
    }

    public function attributes(string|null $namespaceOrPrefix = null, bool $isPrefix = false): null|static
    {
    }

    public function children(string|null $namespaceOrPrefix = null, bool $isPrefix = false): null|static
    {
    }

    /**
     * @return array<string, string>
     */
    public function getNamespaces(bool $recursive = false): array
    {
    }

    /**
     * @return array<string, string>|false
     */
    public function getDocNamespaces(bool $recursive = false, bool $fromRoot = true): array|false
    {
    }

    public function getName(): string
    {
    }

    public function addChild(
        string $qualifiedName,
        string|null $value = null,
        string|null $namespace = null,
    ): null|static {
    }

    public function addAttribute(string $qualifiedName, string $value, string|null $namespace = null): void
    {
    }

    public function __toString(): string
    {
    }

    /**
     * @return int<0, max>
     */
    public function count(): int
    {
    }

    /**
     * @param string|int $offset
     *
     * @return bool
     */
    public function offsetExists($offset)
    {
    }

    /**
     * @param string|int $offset
     *
     * @return static
     */
    public function offsetGet($offset)
    {
    }

    /**
     * @param string|int $offset
     *
     * @param mixed $value
     */
    public function offsetSet($offset, $value)
    {
    }

    /**
     * @param string|int $offset
     *
     * @return void
     */
    public function offsetUnset($offset)
    {
    }

    public function rewind(): void
    {
    }

    public function valid(): bool
    {
    }

    /**
     * @return SimpleXMLElement|null
     */
    public function current(): null|static
    {
    }

    public function key(): string
    {
    }

    public function next(): void
    {
    }

    public function hasChildren(): bool
    {
    }

    public function getChildren(): null|SimpleXMLElement
    {
    }
}

class SimpleXMLIterator extends SimpleXMLElement implements RecursiveIterator, Countable, Stringable
{
    /**
     * @return void
     */
    public function rewind()
    {
    }

    /**
     * @return bool
     */
    public function valid()
    {
    }

    /**
     * @return static|null
     */
    public function current()
    {
    }

    /**
     * @return string|false
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
    public function hasChildren()
    {
    }

    /**
     * @return SimpleXMLIterator
     */
    public function getChildren()
    {
    }

    /**
     * @return string
     */
    public function __toString()
    {
    }

    public function count()
    {
    }
}

/**
 * @template T of SimpleXMLElement
 *
 * @param class-string<T>|null $class_name
 *
 * @return T|false
 */
function simplexml_load_file(
    string $filename,
    null|string $class_name = SimpleXMLElement::class,
    int $options = 0,
    string $namespace_or_prefix = '',
    bool $is_prefix = false,
): SimpleXMLElement|false {
}

/**
 * @template T of SimpleXMLElement
 *
 * @param class-string<T>|null $class_name
 *
 * @return T|false
 */
function simplexml_load_string(
    string $data,
    null|string $class_name = SimpleXMLElement::class,
    int $options = 0,
    string $namespace_or_prefix = '',
    bool $is_prefix = false,
): SimpleXMLElement|false {
}

/**
 * @template T of SimpleXMLElement
 *
 * @param class-string<T>|null $class_name
 *
 * @return T|null
 */
function simplexml_import_dom(object $node, null|string $class_name = SimpleXMLElement::class): null|SimpleXMLElement
{
}
