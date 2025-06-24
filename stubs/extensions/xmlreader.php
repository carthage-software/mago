<?php

class XMLReader
{
    public const int NONE = UNKNOWN;
    public const int ELEMENT = UNKNOWN;
    public const int ATTRIBUTE = UNKNOWN;
    public const int TEXT = UNKNOWN;
    public const int CDATA = UNKNOWN;
    public const int ENTITY_REF = UNKNOWN;
    public const int ENTITY = UNKNOWN;
    public const int PI = UNKNOWN;
    public const int COMMENT = UNKNOWN;
    public const int DOC = UNKNOWN;
    public const int DOC_TYPE = UNKNOWN;
    public const int DOC_FRAGMENT = UNKNOWN;
    public const int NOTATION = UNKNOWN;
    public const int WHITESPACE = UNKNOWN;
    public const int SIGNIFICANT_WHITESPACE = UNKNOWN;
    public const int END_ELEMENT = UNKNOWN;
    public const int END_ENTITY = UNKNOWN;
    public const int XML_DECLARATION = UNKNOWN;
    public const int LOADDTD = UNKNOWN;
    public const int DEFAULTATTRS = UNKNOWN;
    public const int VALIDATE = UNKNOWN;
    public const int SUBST_ENTITIES = UNKNOWN;

    public int $attributeCount;

    public string $baseURI;

    public int $depth;

    public bool $hasAttributes;

    public bool $hasValue;

    public bool $isDefault;

    public bool $isEmptyElement;

    public string $localName;

    public string $name;

    public string $namespaceURI;

    public int $nodeType;

    public string $prefix;

    public string $value;

    public string $xmlLang;

    public function close(): true
    {
    }

    public function getAttribute(string $name): null|string
    {
    }

    public function getAttributeNo(int $index): null|string
    {
    }

    public function getAttributeNs(string $name, string $namespace): null|string
    {
    }

    public function getParserProperty(int $property): bool
    {
    }

    public function isValid(): bool
    {
    }

    public function lookupNamespace(string $prefix): null|string
    {
    }

    public function moveToAttribute(string $name): bool
    {
    }

    public function moveToAttributeNo(int $index): bool
    {
    }

    public function moveToAttributeNs(string $name, string $namespace): bool
    {
    }

    public function moveToElement(): bool
    {
    }

    public function moveToFirstAttribute(): bool
    {
    }

    public function moveToNextAttribute(): bool
    {
    }

    public function read(): bool
    {
    }

    public function next(null|string $name = null): bool
    {
    }

    /** @return bool|XMLReader */
    public static function open(string $uri, null|string $encoding = null, int $flags = 0)
    {
    }

    public static function fromUri(string $uri, null|string $encoding = null, int $flags = 0): static
    {
    }

    /** @param resource $stream */
    public static function fromStream(
        $stream,
        null|string $encoding = null,
        int $flags = 0,
        null|string $documentUri = null,
    ): static {
    }

    public function readInnerXml(): string
    {
    }

    public function readOuterXml(): string
    {
    }

    public function readString(): string
    {
    }

    public function setSchema(null|string $filename): bool
    {
    }

    public function setParserProperty(int $property, bool $value): bool
    {
    }

    public function setRelaxNGSchema(null|string $filename): bool
    {
    }

    public function setRelaxNGSchemaSource(null|string $source): bool
    {
    }

    /** @return bool|XMLReader */
    public static function XML(string $source, null|string $encoding = null, int $flags = 0)
    {
    }

    public static function fromString(string $source, null|string $encoding = null, int $flags = 0): static
    {
    }

    public function expand(null|DOMNode $baseNode = null): DOMNode|false
    {
    }
}
