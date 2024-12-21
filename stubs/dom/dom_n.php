<?php

namespace Dom;

/**
 * @since 8.4
 */
class Implementation
{
    public function createDocumentType(string $qualifiedName, string $publicId, string $systemId): DocumentType {}

    public function createDocument(?string $namespace, string $qualifiedName, ?DocumentType $doctype = null): XMLDocument {}

    public function createHTMLDocument(?string $title = null): HTMLDocument {}
}

/**
 * @since 8.4
 */
final readonly class NamespaceInfo
{
    public readonly ?string $prefix;
    public readonly ?string $namespaceURI;
    public readonly Element $element;

    private function __construct() {}
}

/**
 * @since 8.4
 * @template TNode of Node Should be template-covariant but DocBlock::getTagsByName() does not support it
 * @implements \IteratorAggregate<int, TNode>
 */
class NodeList implements \IteratorAggregate, \Countable
{
    public int $length;

    public function count(): int {}

    /**
     * @return \Iterator<int, TNode>
     */
    public function getIterator(): \Iterator {}

    /** @return TNode|null */
    public function item(int $index): ?Node {}
}
/**
 * @since 8.4
 * @implements \IteratorAggregate<array-key, Attr>
 */
class NamedNodeMap implements \IteratorAggregate, \Countable
{
    public int $length;

    public function item(int $index): ?Attr {}

    public function getNamedItem(string $qualifiedName): ?Attr {}

    public function getNamedItemNS(?string $namespace, string $localName): ?Attr {}

    public function count(): int {}

    /**
     * @return \Iterator<array-key, Attr>
     */
    public function getIterator(): \Iterator {}
}
/**
 * @since 8.4
 *
 * @template TDtdNode of Entity|Notation Should be template-covariant but DocBlock::getTagsByName() does not support it
 * @implements \IteratorAggregate<string, TDtdNode>
 */
class DtdNamedNodeMap implements \IteratorAggregate, \Countable
{
    public int $length;

    /** @return TDtdNode|null */
    public function item(int $index): Entity|Notation|null {}

    /** @return TDtdNode|null */
    public function getNamedItem(string $qualifiedName): Entity|Notation|null {}

    /** @return TDtdNode|null */
    public function getNamedItemNS(?string $namespace, string $localName): Entity|Notation|null {}

    public function count(): int {}

    /** @return \Iterator<string, TDtdNode> */
    public function getIterator(): \Iterator {}
}
/**
 * @since 8.4
 * @implements \IteratorAggregate<array-key, Element>
 */
class HTMLCollection implements \IteratorAggregate, \Countable
{
    public int $length;

    public function item(int $index): ?Element {}

    public function namedItem(string $key): ?Element {}

    public function count(): int {}

    /**
     * @return \Iterator<array-key, Element>
     */
    public function getIterator(): \Iterator {}
}
/**
 * @since 8.4
 */
final class XPath
{
    public Document $document;
    public bool $registerNodeNamespaces;

    public function __construct(Document $document, bool $registerNodeNS = true) {}

    /** @return null|bool|float|string|NodeList<Node> */
    public function evaluate(string $expression, ?Node $contextNode = null, bool $registerNodeNS = true): null|bool|float|string|NodeList {}

    /** @return NodeList<Node> */
    public function query(string $expression, ?Node $contextNode = null, bool $registerNodeNS = true): NodeList {}

    public function registerNamespace(string $prefix, string $namespace): bool {}

    public function registerPhpFunctions(string|array|null $restrict = null): void {}

    public function registerPhpFunctionNS(string $namespaceURI, string $name, callable $callable): void {}

    public static function quote(string $str): string {}
}
/**
 * @since 8.4
 */
final class TokenList implements \IteratorAggregate, \Countable
{
    private function __construct() {}
    public int $length;

    public function item(int $index): ?string {}

    public function contains(string $token): bool {}

    public function add(string ...$tokens): void {}

    public function remove(string ...$tokens): void {}

    public function toggle(string $token, ?bool $force = null): bool {}

    public function replace(string $token, string $newToken): bool {}

    public function supports(string $token): bool {}
    public string $value;

    public function count(): int {}

    public function getIterator(): \Iterator {}
}
/**
 * @since 8.4
 */
interface ParentNode
{
    public function append(Node|string ...$nodes): void;

    public function prepend(Node|string ...$nodes): void;

    public function replaceChildren(Node|string ...$nodes): void;

    public function querySelector(string $selectors): ?Element;

    /** @return NodeList<Element> */
    public function querySelectorAll(string $selectors): NodeList;
}
/**
 * @since 8.4
 */
interface ChildNode
{
    public function remove(): void;

    public function before(Node|string ...$nodes): void;

    public function after(Node|string ...$nodes): void;

    public function replaceWith(Node|string ...$nodes): void;
}
/**
 * @since 8.4
 */
enum AdjacentPosition implements \BackedEnum, \UnitEnum
{
    case BeforeBegin = "beforebegin";
    case AfterBegin = "afterbegin";
    case BeforeEnd = "beforeend";
    case AfterEnd = "afterend";

    public static function cases(): array {}

    public static function from(string|int $value): static {}

    public static function tryFrom(string|int $value): ?static {}
}
/**
 * @since 8.4
 */
class Node
{
    final private function __construct() {}
    public int $nodeType;
    public string $nodeName;
    public string $baseURI;
    public bool $isConnected;
    public ?Document $ownerDocument;

    public function getRootNode(array $options = []): Node {}
    public ?Node $parentNode;
    public ?Element $parentElement;

    public function hasChildNodes(): bool {}

    /** @var NodeList<Node> */
    public NodeList $childNodes;
    public ?Node $firstChild;
    public ?Node $lastChild;
    public ?Node $previousSibling;
    public ?Node $nextSibling;
    public ?string $nodeValue;
    public ?string $textContent;

    public function normalize(): void {}

    public function cloneNode(bool $deep = false): Node {}

    public function isEqualNode(?Node $otherNode): bool {}

    public function isSameNode(?Node $otherNode): bool {}
    public const int DOCUMENT_POSITION_DISCONNECTED = 0x01;
    public const int DOCUMENT_POSITION_PRECEDING = 0x02;
    public const int DOCUMENT_POSITION_FOLLOWING = 0x04;
    public const int DOCUMENT_POSITION_CONTAINS = 0x08;
    public const int DOCUMENT_POSITION_CONTAINED_BY = 0x10;
    public const int DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 0x20;

    public function compareDocumentPosition(Node $other): int {}

    public function contains(?Node $other): bool {}

    public function lookupPrefix(?string $namespace): ?string {}

    public function lookupNamespaceURI(?string $prefix): ?string {}

    public function isDefaultNamespace(?string $namespace): bool {}

    public function insertBefore(Node $node, ?Node $child): Node {}

    public function appendChild(Node $node): Node {}

    public function replaceChild(Node $node, Node $child): Node {}

    public function removeChild(Node $child): Node {}

    public function getLineNo(): int {}

    public function getNodePath(): string {}

    public function C14N(bool $exclusive = false, bool $withComments = false, ?array $xpath = null, ?array $nsPrefixes = null): string|false {}

    public function C14NFile(string $uri, bool $exclusive = false, bool $withComments = false, ?array $xpath = null, ?array $nsPrefixes = null): int|false {}

    public function __sleep(): array {}

    public function __wakeup(): void {}
}
/**
 * @since 8.4
 */
class DocumentFragment extends Node implements ParentNode
{
    public ?Element $firstElementChild;
    public ?Element $lastElementChild;
    public int $childElementCount;

    public function appendXml(string $data): bool {}

    public function append(Node|string ...$nodes): void {}

    public function prepend(Node|string ...$nodes): void {}

    public function replaceChildren(Node|string ...$nodes): void {}

    public function querySelector(string $selectors): ?Element {}

    /** @return NodeList<Element> */
    public function querySelectorAll(string $selectors): NodeList {}
}
/**
 * @since 8.4
 */
class Document extends Node implements ParentNode
{
    /** @readonly */
    public Implementation $implementation;
    public string $URL;
    public string $documentURI;
    public string $characterSet;
    public string $charset;
    public string $inputEncoding;
    public ?DocumentType $doctype;
    public ?Element $documentElement;

    public function getElementsByTagName(string $qualifiedName): HTMLCollection {}

    public function getElementsByTagNameNS(?string $namespace, string $localName): HTMLCollection {}

    public function createElement(string $localName): Element {}

    public function createElementNS(?string $namespace, string $qualifiedName): Element {}

    public function createDocumentFragment(): DocumentFragment {}

    public function createTextNode(string $data): Text {}

    public function createCDATASection(string $data): CDATASection {}

    public function createComment(string $data): Comment {}

    public function createProcessingInstruction(string $target, string $data): ProcessingInstruction {}

    public function importNode(?Node $node, bool $deep = false): Node {}

    public function adoptNode(Node $node): Node {}

    public function createAttribute(string $localName): Attr {}

    public function createAttributeNS(?string $namespace, string $qualifiedName): Attr {}
    public ?Element $firstElementChild;
    public ?Element $lastElementChild;
    public int $childElementCount;

    public function getElementById(string $elementId): ?Element {}

    public function registerNodeClass(string $baseClass, ?string $extendedClass): void {}

    public function schemaValidate(string $filename, int $flags = 0): bool {}

    public function schemaValidateSource(string $source, int $flags = 0): bool {}

    public function relaxNgValidate(string $filename): bool {}

    public function relaxNgValidateSource(string $source): bool {}

    public function append(Node|string ...$nodes): void {}

    public function prepend(Node|string ...$nodes): void {}

    public function replaceChildren(Node|string ...$nodes): void {}

    public function importLegacyNode(\DOMNode $node, bool $deep = false): Node {}

    public function querySelector(string $selectors): ?Element {}

    /** @return NodeList<Element> */
    public function querySelectorAll(string $selectors): NodeList {}
    public ?HTMLElement $body;
    public ?HTMLElement $head;
    public string $title;
}
/**
 * @since 8.4
 */
final class HTMLDocument extends Document
{
    public static function createEmpty(string $encoding = "UTF-8"): HTMLDocument {}

    public static function createFromFile(string $path, int $options = 0, ?string $overrideEncoding = null): HTMLDocument {}

    public static function createFromString(string $source, int $options = 0, ?string $overrideEncoding = null): HTMLDocument {}

    public function saveXml(?Node $node = null, int $options = 0): string|false {}

    public function saveXmlFile(string $filename, int $options = 0): int|false {}

    public function saveHtml(?Node $node = null): string {}

    public function saveHtmlFile(string $filename): int|false {}

    public function debugGetTemplateCount(): int {}
}
/**
 * @since 8.4
 */
final class XMLDocument extends Document
{
    public static function createEmpty(string $version = "1.0", string $encoding = "UTF-8"): XMLDocument {}

    public static function createFromFile(string $path, int $options = 0, ?string $overrideEncoding = null): XMLDocument {}

    public static function createFromString(string $source, int $options = 0, ?string $overrideEncoding = null): XMLDocument {}
    public string $xmlEncoding;
    public bool $xmlStandalone;
    public string $xmlVersion;
    public bool $formatOutput;

    public function createEntityReference(string $name): EntityReference {}

    public function validate(): bool {}

    public function xinclude(int $options = 0): int {}

    public function saveXml(?Node $node = null, int $options = 0): string|false {}

    public function saveXmlFile(string $filename, int $options = 0): int|false {}
}
/**
 * @since 8.4
 */
class CharacterData extends Node implements ChildNode
{
    public ?Element $previousElementSibling;
    public ?Element $nextElementSibling;
    public string $data;
    public int $length;

    public function substringData(int $offset, int $count): string {}

    public function appendData(string $data): void {}

    public function insertData(int $offset, string $data): void {}

    public function deleteData(int $offset, int $count): void {}

    public function replaceData(int $offset, int $count, string $data): void {}

    public function remove(): void {}

    public function before(Node|string ...$nodes): void {}

    public function after(Node|string ...$nodes): void {}

    public function replaceWith(Node|string ...$nodes): void {}
}
/**
 * @since 8.4
 */
class Attr extends Node
{
    public ?string $namespaceURI;
    public ?string $prefix;
    public string $localName;
    public string $name;
    public string $value;
    public ?Element $ownerElement;
    public bool $specified;

    public function isId(): bool {}

    public function rename(?string $namespaceURI, string $qualifiedName): void {}
}
/**
 * @since 8.4
 */
class Element extends Node implements ParentNode, ChildNode
{
    public ?string $namespaceURI;
    public ?string $prefix;
    public string $localName;
    public string $tagName;
    public string $id;
    public string $className;

    /** @readonly */
    public TokenList $classList;

    public function hasAttributes(): bool {}
    public NamedNodeMap $attributes;

    public function getAttributeNames(): array {}

    public function getAttribute(string $qualifiedName): ?string {}

    public function getAttributeNS(?string $namespace, string $localName): ?string {}

    public function setAttribute(string $qualifiedName, string $value): void {}

    public function setAttributeNS(?string $namespace, string $qualifiedName, string $value): void {}

    public function removeAttribute(string $qualifiedName): void {}

    public function removeAttributeNS(?string $namespace, string $localName): void {}

    public function toggleAttribute(string $qualifiedName, ?bool $force = null): bool {}

    public function hasAttribute(string $qualifiedName): bool {}

    public function hasAttributeNS(?string $namespace, string $localName): bool {}

    public function getAttributeNode(string $qualifiedName): ?Attr {}

    public function getAttributeNodeNS(?string $namespace, string $localName): ?Attr {}

    public function setAttributeNode(Attr $attr): ?Attr {}

    public function setAttributeNodeNS(Attr $attr): ?Attr {}

    public function removeAttributeNode(Attr $attr): Attr {}

    public function getElementsByTagName(string $qualifiedName): HTMLCollection {}

    public function getElementsByTagNameNS(?string $namespace, string $localName): HTMLCollection {}

    public function insertAdjacentElement(AdjacentPosition $where, Element $element): ?Element {}

    public function insertAdjacentText(AdjacentPosition $where, string $data): void {}
    public ?Element $firstElementChild;
    public ?Element $lastElementChild;
    public int $childElementCount;
    public ?Element $previousElementSibling;
    public ?Element $nextElementSibling;

    public function setIdAttribute(string $qualifiedName, bool $isId): void {}

    public function setIdAttributeNS(?string $namespace, string $qualifiedName, bool $isId): void {}

    public function setIdAttributeNode(Attr $attr, bool $isId): void {}

    public function remove(): void {}

    public function before(Node|string ...$nodes): void {}

    public function after(Node|string ...$nodes): void {}

    public function replaceWith(Node|string ...$nodes): void {}

    public function append(Node|string ...$nodes): void {}

    public function prepend(Node|string ...$nodes): void {}

    public function replaceChildren(Node|string ...$nodes): void {}

    public function querySelector(string $selectors): ?Element {}

    /** @return NodeList<Element> */
    public function querySelectorAll(string $selectors): NodeList {}

    public function closest(string $selectors): ?Element {}

    public function matches(string $selectors): bool {}
    public string $innerHTML;
    public string $outerHTML;
    public string $substitutedNodeValue;

    /** @return list<NamespaceInfo> */
    public function getInScopeNamespaces(): array {}

    /** @return list<NamespaceInfo> */
    public function getDescendantNamespaces(): array {}

    public function rename(?string $namespaceURI, string $qualifiedName): void {}
}
/**
 * @since 8.4
 */
class HTMLElement extends Element {}
/**
 * @since 8.4
 */
class Text extends CharacterData
{
    public function splitText(int $offset): Text {}
    public string $wholeText;
}
/**
 * @since 8.4
 */
class Comment extends CharacterData {}
/**
 * @since 8.4
 */
class CDATASection extends Text {}
/**
 * @since 8.4
 */
class DocumentType extends Node implements ChildNode
{
    public string $name;
    public DtdNamedNodeMap $entities;
    public DtdNamedNodeMap $notations;
    public string $publicId;
    public string $systemId;
    public ?string $internalSubset;

    public function remove(): void {}

    public function before(Node|string ...$nodes): void {}

    public function after(Node|string ...$nodes): void {}

    public function replaceWith(Node|string ...$nodes): void {}
}
/**
 * @since 8.4
 */
class Notation extends Node
{
    public string $publicId;
    public string $systemId;
}
/**
 * @since 8.4
 */
class Entity extends Node
{
    public ?string $publicId;
    public ?string $systemId;
    public ?string $notationName;
}
/**
 * @since 8.4
 */
class EntityReference extends Node {}
/**
 * @since 8.4
 */
class ProcessingInstruction extends CharacterData
{
    public string $target;
}
/**
 * @since 8.4
 */
class Sqlite {}
/**
 * @since 8.4
 */
class RandomError {}
/**
 * @since 8.4
 */
class BrokenRandomEngineError {}
/**
 * @since 8.4
 */
class RandomException {}
/**
 * @since 8.4
 */
class Mysql {}
/**
 * @since 8.4
 */
function import_simplexml(object $node): Attr|Element {}

/**
 * @since 8.4
 */
const INDEX_SIZE_ERR = 1, STRING_SIZE_ERR = 2, HIERARCHY_REQUEST_ERR = 3, WRONG_DOCUMENT_ERR = 4, INVALID_CHARACTER_ERR = 5;
/**
 * @since 8.4
 */
const NO_DATA_ALLOWED_ERR = 6, NO_MODIFICATION_ALLOWED_ERR = 7, NOT_FOUND_ERR = 8, NOT_SUPPORTED_ERR = 9, INUSE_ATTRIBUTE_ERR = 10;
/**
 * @since 8.4
 */
const INVALID_STATE_ERR = 11, SYNTAX_ERR = 12, INVALID_MODIFICATION_ERR = 13, NAMESPACE_ERR = 14, VALIDATION_ERR = 16, HTML_NO_DEFAULT_NS = 2147483648;
