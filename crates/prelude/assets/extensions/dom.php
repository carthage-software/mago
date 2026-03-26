<?php

namespace {
    /**
     * @var int
     */
    const XML_ELEMENT_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_TEXT_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_CDATA_SECTION_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_ENTITY_REF_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_ENTITY_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_PI_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_COMMENT_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_DOCUMENT_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_DOCUMENT_TYPE_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_DOCUMENT_FRAG_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_NOTATION_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_HTML_DOCUMENT_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_DTD_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_ELEMENT_DECL_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_DECL_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_ENTITY_DECL_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_NAMESPACE_DECL_NODE = UNKNOWN;

    /**
     * @var int
     */
    const XML_LOCAL_NAMESPACE = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_CDATA = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_ID = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_IDREF = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_IDREFS = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_ENTITY = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_NMTOKEN = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_NMTOKENS = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_ENUMERATION = UNKNOWN;

    /**
     * @var int
     */
    const XML_ATTRIBUTE_NOTATION = UNKNOWN;

    /**
     * @var int
     */
    const DOM_PHP_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_INDEX_SIZE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOMSTRING_SIZE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_HIERARCHY_REQUEST_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_WRONG_DOCUMENT_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_INVALID_CHARACTER_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_NO_DATA_ALLOWED_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_NO_MODIFICATION_ALLOWED_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_NOT_FOUND_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_NOT_SUPPORTED_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_INUSE_ATTRIBUTE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_INVALID_STATE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_SYNTAX_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_INVALID_MODIFICATION_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_NAMESPACE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_INVALID_ACCESS_ERR = UNKNOWN;

    /**
     * @var int
     */
    const DOM_VALIDATION_ERR = UNKNOWN;

    class DOMDocumentType extends DOMNode
    {
        /**
         * @readonly
         */
        public string $name;

        /**
         * @readonly
         */
        public DOMNamedNodeMap $entities;

        /**
         * @readonly
         */
        public DOMNamedNodeMap $notations;

        /**
         * @readonly
         */
        public string $publicId;

        /**
         * @readonly
         */
        public string $systemId;

        /**
         * @readonly
         */
        public ?string $internalSubset;
    }

    class DOMCdataSection extends DOMText
    {
        public function __construct(string $data) {}
    }

    class DOMComment extends DOMCharacterData
    {
        public function __construct(string $data = '') {}
    }

    interface DOMParentNode
    {
        /** @param DOMNode|string $nodes */
        public function append(...$nodes): void;

        /** @param DOMNode|string $nodes */
        public function prepend(...$nodes): void;

        /** @param DOMNode|string $nodes */
        public function replaceChildren(...$nodes): void;
    }

    interface DOMChildNode
    {
        public function remove(): void;

        /** @param DOMNode|string $nodes */
        public function before(...$nodes): void;

        /** @param DOMNode|string $nodes */
        public function after(...$nodes): void;

        /** @param DOMNode|string $nodes */
        public function replaceWith(...$nodes): void;
    }

    class DOMNode
    {
        public const int DOCUMENT_POSITION_DISCONNECTED = 0x01;
        public const int DOCUMENT_POSITION_PRECEDING = 0x02;
        public const int DOCUMENT_POSITION_FOLLOWING = 0x04;
        public const int DOCUMENT_POSITION_CONTAINS = 0x08;
        public const int DOCUMENT_POSITION_CONTAINED_BY = 0x10;
        public const int DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 0x20;

        /**
         * @readonly
         */
        public string $nodeName;

        public ?string $nodeValue;

        /**
         * @readonly
         */
        public int $nodeType;

        /**
         * @readonly
         */
        public ?DOMNode $parentNode;

        /**
         * @readonly
         */
        public ?DOMElement $parentElement;

        /**
         * @readonly
         */
        public DOMNodeList $childNodes;

        /**
         * @readonly
         */
        public ?DOMNode $firstChild;

        /**
         * @readonly
         */
        public ?DOMNode $lastChild;

        /**
         * @readonly
         */
        public ?DOMNode $previousSibling;

        /**
         * @readonly
         */
        public ?DOMNode $nextSibling;

        /**
         * @readonly
         * @var DOMNamedNodeMap<DOMAttr>
         */
        public ?DOMNamedNodeMap $attributes;

        /**
         * @readonly
         */
        public bool $isConnected;

        /**
         * @readonly
         */
        public ?DOMDocument $ownerDocument;

        /**
         * @readonly
         */
        public ?string $namespaceURI;

        public string $prefix;

        /**
         * @readonly
         */
        public ?string $localName;

        /**
         * @readonly
         */
        public ?string $baseURI;

        public string $textContent;

        /** @return DOMNode|false */
        public function appendChild(DOMNode $node) {}

        public function C14N(
            bool $exclusive = false,
            bool $withComments = false,
            ?array $xpath = null,
            ?array $nsPrefixes = null,
        ): string|false {}

        public function C14NFile(
            string $uri,
            bool $exclusive = false,
            bool $withComments = false,
            ?array $xpath = null,
            ?array $nsPrefixes = null,
        ): int|false {}

        /** @return DOMNode|false */
        public function cloneNode(bool $deep = false) {}

        public function getLineNo(): int {}

        public function getNodePath(): ?string {}

        public function hasAttributes(): bool {}

        public function hasChildNodes(): bool {}

        /** @return DOMNode|false */
        public function insertBefore(DOMNode $node, ?DOMNode $child = null) {}

        public function isDefaultNamespace(string $namespace): bool {}

        public function isSameNode(DOMNode $otherNode): bool {}

        public function isEqualNode(?DOMNode $otherNode): bool {}

        public function isSupported(string $feature, string $version): bool {}

        public function lookupNamespaceURI(?string $prefix): ?string {}

        public function lookupPrefix(string $namespace): ?string {}

        public function normalize(): void {}

        /** @return DOMNode|false */
        public function removeChild(DOMNode $child) {}

        /** @return DOMNode|false */
        public function replaceChild(DOMNode $node, DOMNode $child) {}

        public function contains(DOMNode|DOMNameSpaceNode|null $other): bool {}

        public function getRootNode(?array $options = null): DOMNode {}

        public function compareDocumentPosition(DOMNode $other): int {}

        public function __sleep(): array {}

        public function __wakeup(): void {}
    }

    class DOMNameSpaceNode
    {
        /**
         * @readonly
         */
        public string $nodeName;

        /**
         * @readonly
         */
        public ?string $nodeValue;

        /**
         * @readonly
         */
        public int $nodeType;

        /**
         * @readonly
         */
        public string $prefix;

        /**
         * @readonly
         */
        public ?string $localName;

        /**
         * @readonly
         */
        public ?string $namespaceURI;

        /**
         * @readonly
         */
        public bool $isConnected;

        /**
         * @readonly
         */
        public ?DOMDocument $ownerDocument;

        /**
         * @readonly
         */
        public ?DOMNode $parentNode;

        /**
         * @readonly
         */
        public ?DOMElement $parentElement;

        public function __sleep(): array {}

        public function __wakeup(): void {}
    }

    class DOMImplementation
    {
        public function getFeature(string $feature, string $version): never {}

        public function hasFeature(string $feature, string $version): bool {}

        /** @return DOMDocumentType|false */
        public function createDocumentType(string $qualifiedName, string $publicId = '', string $systemId = '') {}

        public function createDocument(
            ?string $namespace = null,
            string $qualifiedName = '',
            ?DOMDocumentType $doctype = null,
        ): DOMDocument {}
    }

    class DOMDocumentFragment extends DOMNode implements DOMParentNode
    {
        /**
         * @readonly
         */
        public ?DOMElement $firstElementChild;

        /**
         * @readonly
         */
        public ?DOMElement $lastElementChild;

        /**
         * @readonly
         */
        public int $childElementCount;

        public function __construct() {}

        public function appendXML(string $data): bool {}

        /**
         * @param DOMNode|string $nodes
         */
        public function append(...$nodes): void {}

        /**
         * @param DOMNode|string $nodes
         */
        public function prepend(...$nodes): void {}

        /**
         * @param DOMNode|string $nodes
         */
        public function replaceChildren(...$nodes): void {}
    }

    /**
     * @template-covariant TNode of DOMNode|DOMNameSpaceNode
     *
     * @implements IteratorAggregate<int, TNode>
     */
    class DOMNodeList implements IteratorAggregate, Countable
    {
        /**
         * @readonly
         */
        public int $length;

        public function count(): int {}

        /**
         * @return Iterator<int, TNode>
         */
        public function getIterator(): Iterator {}

        /**
         * @return TNode|null
         */
        public function item(int $index) {}
    }

    class DOMCharacterData extends DOMNode implements DOMChildNode
    {
        public string $data;

        /**
         * @readonly
         */
        public int $length;

        /**
         * @readonly
         */
        public ?DOMElement $previousElementSibling;

        /**
         * @readonly
         */
        public ?DOMElement $nextElementSibling;

        public function appendData(string $data): true {}

        /** @return string|false */
        public function substringData(int $offset, int $count) {}

        public function insertData(int $offset, string $data): bool {}

        public function deleteData(int $offset, int $count): bool {}

        public function replaceData(int $offset, int $count, string $data): bool {}

        /**
         * @param DOMNode|string $nodes
         */
        public function replaceWith(...$nodes): void {}

        public function remove(): void {}

        /**
         * @param DOMNode|string $nodes
         */
        public function before(...$nodes): void {}

        /**
         * @param DOMNode|string $nodes
         */
        public function after(...$nodes): void {}
    }

    class DOMAttr extends DOMNode
    {
        /**
         * @readonly
         */
        public string $name;

        /**
         * @readonly
         */
        public bool $specified = true;

        public string $value;

        /**
         * @readonly
         */
        public ?DOMElement $ownerElement;

        /**
         * @readonly
         */
        public mixed $schemaTypeInfo = null;

        public function __construct(string $name, string $value = '') {}

        public function isId(): bool {}
    }

    class DOMElement extends DOMNode implements \DOMParentNode, \DOMChildNode
    {
        /**
         * @readonly
         */
        public string $tagName;

        public string $className;

        public string $id;

        /**
         * @readonly
         */
        public mixed $schemaTypeInfo = null;

        /**
         * @readonly
         */
        public ?DOMElement $firstElementChild;

        /**
         * @readonly
         */
        public ?DOMElement $lastElementChild;

        /**
         * @readonly
         */
        public int $childElementCount;

        /**
         * @readonly
         */
        public ?DOMElement $previousElementSibling;

        /**
         * @readonly
         */
        public ?DOMElement $nextElementSibling;

        public function __construct(string $qualifiedName, ?string $value = null, string $namespace = '') {}

        public function getAttribute(string $qualifiedName): string {}

        public function getAttributeNames(): array {}

        public function getAttributeNS(?string $namespace, string $localName): string {}

        /** @return DOMAttr|DOMNameSpaceNode|false */
        public function getAttributeNode(string $qualifiedName) {}

        /** @return DOMAttr|DOMNameSpaceNode|null */
        public function getAttributeNodeNS(?string $namespace, string $localName) {}

        /**
         * @return DOMNodeList<DOMElement>
         */
        public function getElementsByTagName(string $qualifiedName): DOMNodeList {}

        /**
         * @return DOMNodeList<DOMElement>
         */
        public function getElementsByTagNameNS(?string $namespace, string $localName): DOMNodeList {}

        public function hasAttribute(string $qualifiedName): bool {}

        public function hasAttributeNS(?string $namespace, string $localName): bool {}

        public function removeAttribute(string $qualifiedName): bool {}

        public function removeAttributeNS(?string $namespace, string $localName): void {}

        /** @return DOMAttr|false */
        public function removeAttributeNode(DOMAttr $attr) {}

        /** @return DOMAttr|bool */
        public function setAttribute(string $qualifiedName, string $value) {}

        public function setAttributeNS(?string $namespace, string $qualifiedName, string $value): void {}

        /** @return DOMAttr|null|false */
        public function setAttributeNode(DOMAttr $attr) {}

        /** @return DOMAttr|null|false */
        public function setAttributeNodeNS(DOMAttr $attr) {}

        public function setIdAttribute(string $qualifiedName, bool $isId): void {}

        public function setIdAttributeNS(string $namespace, string $qualifiedName, bool $isId): void {}

        public function setIdAttributeNode(DOMAttr $attr, bool $isId): void {}

        public function toggleAttribute(string $qualifiedName, ?bool $force = null): bool {}

        public function remove(): void {}

        /** @param DOMNode|string $nodes */
        public function before(...$nodes): void {}

        /** @param DOMNode|string $nodes */
        public function after(...$nodes): void {}

        /** @param DOMNode|string $nodes */
        public function replaceWith(...$nodes): void {}

        /** @param DOMNode|string $nodes */
        public function append(...$nodes): void {}

        /** @param DOMNode|string $nodes */
        public function prepend(...$nodes): void {}

        /** @param DOMNode|string $nodes */
        public function replaceChildren(...$nodes): void {}

        public function insertAdjacentElement(string $where, DOMElement $element): ?DOMElement {}

        public function insertAdjacentText(string $where, string $data): void {}
    }

    class DOMDocument extends DOMNode implements DOMParentNode
    {
        /**
         * @readonly
         */
        public ?DOMDocumentType $doctype;

        /**
         * @readonly
         */
        public DOMImplementation $implementation;

        /**
         * @readonly
         */
        public ?DOMElement $documentElement;

        /**
         * @readonly
         * @deprecated
         */
        public ?string $actualEncoding;

        public ?string $encoding;

        /**
         * @readonly
         */
        public ?string $xmlEncoding;

        public bool $standalone;

        public bool $xmlStandalone;

        public ?string $version;

        public ?string $xmlVersion;

        public bool $strictErrorChecking;

        public ?string $documentURI;

        /**
         * @readonly
         * @deprecated
         */
        public mixed $config;

        public bool $formatOutput;

        public bool $validateOnParse;

        public bool $resolveExternals;

        public bool $preserveWhiteSpace;

        public bool $recover;

        public bool $substituteEntities;

        /**
         * @readonly
         */
        public ?DOMElement $firstElementChild;

        /**
         * @readonly
         */
        public ?DOMElement $lastElementChild;

        /**
         * @readonly
         */
        public int $childElementCount;

        public function __construct(string $version = '1.0', string $encoding = '') {}

        /** @return DOMAttr|false */
        public function createAttribute(string $localName) {}

        /** @return DOMAttr|false */
        public function createAttributeNS(?string $namespace, string $qualifiedName) {}

        /** @return DOMCdataSection|false */
        public function createCDATASection(string $data) {}

        public function createComment(string $data): DOMComment {}

        public function createDocumentFragment(): DOMDocumentFragment {}

        /** @return DOMElement|false */
        public function createElement(string $localName, string $value = '') {}

        /** @return DOMElement|false */
        public function createElementNS(?string $namespace, string $qualifiedName, string $value = '') {}

        /** @return DOMEntityReference|false */
        public function createEntityReference(string $name) {}

        /** @return DOMProcessingInstruction|false */
        public function createProcessingInstruction(string $target, string $data = '') {}

        public function createTextNode(string $data): DOMText {}

        public function getElementById(string $elementId): ?DOMElement {}

        /**
         * @return DOMNodeList<DOMElement>
         */
        public function getElementsByTagName(string $qualifiedName): DOMNodeList {}

        /**
         * @return DOMNodeList<DOMElement>
         */
        public function getElementsByTagNameNS(?string $namespace, string $localName): DOMNodeList {}

        /** @return DOMNode|false */
        public function importNode(DOMNode $node, bool $deep = false) {}

        public function load(string $filename, int $options = 0): bool {}

        public function loadXML(string $source, int $options = 0): bool {}

        public function normalizeDocument(): void {}

        public function registerNodeClass(string $baseClass, ?string $extendedClass): true {}

        public function save(string $filename, int $options = 0): int|false {}

        public function loadHTML(string $source, int $options = 0): bool {}

        public function loadHTMLFile(string $filename, int $options = 0): bool {}

        public function saveHTML(?DOMNode $node = null): string|false {}

        public function saveHTMLFile(string $filename): int|false {}

        public function saveXML(?DOMNode $node = null, int $options = 0): string|false {}

        public function schemaValidate(string $filename, int $flags = 0): bool {}

        public function schemaValidateSource(string $source, int $flags = 0): bool {}

        public function relaxNGValidate(string $filename): bool {}

        public function relaxNGValidateSource(string $source): bool {}

        public function validate(): bool {}

        public function xinclude(int $options = 0): int|false {}

        public function adoptNode(DOMNode $node): DOMNode|false {}

        /**
         * @param DOMNode|string $nodes
         */
        public function append(...$nodes): void {}

        /**
         * @param DOMNode|string $nodes
         */
        public function prepend(...$nodes): void {}

        /** @param DOMNode|string $nodes */
        public function replaceChildren(...$nodes): void {}
    }

    final class DOMException extends Exception
    {
        /**
         * @var int
         */
        public $code = 0;
    }

    class DOMText extends DOMCharacterData
    {
        /**
         * @readonly
         */
        public string $wholeText;

        public function __construct(string $data = '') {}

        public function isWhitespaceInElementContent(): bool {}

        public function isElementContentWhitespace(): bool {}

        /** @return DOMText|false */
        public function splitText(int $offset) {}
    }

    /**
     * @template-covariant TNode of DOMNode
     * @implements IteratorAggregate<string, TNode>
     */
    class DOMNamedNodeMap implements IteratorAggregate, Countable
    {
        /**
         * @readonly
         */
        public int $length;

        /** @return null|TNode */
        public function getNamedItem(string $qualifiedName): ?DOMNode {}

        /** @return null|TNode */
        public function getNamedItemNS(?string $namespace, string $localName): ?DOMNode {}

        /** @return null|TNode */
        public function item(int $index): ?DOMNode {}

        public function count(): int {}

        /**
         * @return Iterator<string, TNode>
         */
        public function getIterator(): Iterator {}
    }

    class DOMEntity extends DOMNode
    {
        /**
         * @readonly
         */
        public ?string $publicId;

        /**
         * @readonly
         */
        public ?string $systemId;

        /**
         * @readonly
         */
        public ?string $notationName;

        /**
         * @readonly
         * @deprecated
         */
        public ?string $actualEncoding = null;

        /**
         * @readonly
         * @deprecated
         */
        public ?string $encoding = null;

        /**
         * @readonly
         * @deprecated
         */
        public ?string $version = null;
    }

    class DOMEntityReference extends DOMNode
    {
        public function __construct(string $name) {}
    }

    class DOMNotation extends DOMNode
    {
        /**
         * @readonly
         */
        public string $publicId;

        /**
         * @readonly
         */
        public string $systemId;
    }

    class DOMProcessingInstruction extends DOMNode
    {
        /**
         * @readonly
         */
        public string $target;

        public string $data;

        public function __construct(string $name, string $value = '') {}
    }

    class DOMXPath
    {
        /**
         * @readonly
         */
        public DOMDocument $document;

        public bool $registerNodeNamespaces;

        public function __construct(DOMDocument $document, bool $registerNodeNS = true) {}

        public function evaluate(
            string $expression,
            ?DOMNode $contextNode = null,
            bool $registerNodeNS = true,
        ): mixed {}

        public function query(string $expression, ?DOMNode $contextNode = null, bool $registerNodeNS = true): mixed {}

        public function registerNamespace(string $prefix, string $namespace): bool {}

        public function registerPhpFunctions(string|array|null $restrict = null): void {}

        public function registerPhpFunctionNS(string $namespaceURI, string $name, callable $callable): void {}

        public static function quote(string $str): string {}
    }

    function dom_import_simplexml(object $node): DOMElement|DOMAttr {}
}

namespace Dom {
    use Countable;
    use Iterator;
    use IteratorAggregate;

    /**
     * @var int
     */
    const INDEX_SIZE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const STRING_SIZE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const HIERARCHY_REQUEST_ERR = UNKNOWN;

    /**
     * @var int
     */
    const WRONG_DOCUMENT_ERR = UNKNOWN;

    /**
     * @var int
     */
    const INVALID_CHARACTER_ERR = UNKNOWN;

    /**
     * @var int
     */
    const NO_DATA_ALLOWED_ERR = UNKNOWN;

    /**
     * @var int
     */
    const NO_MODIFICATION_ALLOWED_ERR = UNKNOWN;

    /**
     * @var int
     */
    const NOT_FOUND_ERR = UNKNOWN;

    /**
     * @var int
     */
    const NOT_SUPPORTED_ERR = UNKNOWN;

    /**
     * @var int
     */
    const INUSE_ATTRIBUTE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const INVALID_STATE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const SYNTAX_ERR = UNKNOWN;

    /**
     * @var int
     */
    const INVALID_MODIFICATION_ERR = UNKNOWN;

    /**
     * @var int
     */
    const NAMESPACE_ERR = UNKNOWN;

    /**
     * @var int
     */
    const VALIDATION_ERR = UNKNOWN;

    /**
     * @var int
     */
    const HTML_NO_DEFAULT_NS = UNKNOWN;

    interface ParentNode
    {
        public function append(Node|string ...$nodes): void;

        public function prepend(Node|string ...$nodes): void;

        public function replaceChildren(Node|string ...$nodes): void;

        public function querySelector(string $selectors): ?Element;

        public function querySelectorAll(string $selectors): NodeList;
    }

    interface ChildNode
    {
        public function remove(): void;

        public function before(Node|string ...$nodes): void;

        public function after(Node|string ...$nodes): void;

        public function replaceWith(Node|string ...$nodes): void;
    }

    /**
     * @strict-properties
     */
    class Implementation
    {
        public function createDocumentType(string $qualifiedName, string $publicId, string $systemId): DocumentType {}

        public function createDocument(
            ?string $namespace,
            string $qualifiedName,
            ?DocumentType $doctype = null,
        ): XMLDocument {}

        public function createHTMLDocument(?string $title = null): HTMLDocument {}
    }

    /** @strict-properties */
    class Node
    {
        final private function __construct() {}

        /**
         * @readonly
         */
        public int $nodeType;

        /**
         * @readonly
         */
        public string $nodeName;

        /**
         * @readonly
         */
        public string $baseURI;

        /**
         * @readonly
         */
        public bool $isConnected;

        /**
         * @readonly
         */
        public ?Document $ownerDocument;

        public function getRootNode(array $options = []): Node {}

        /**
         * @readonly
         */
        public ?Node $parentNode;

        /**
         * @readonly
         */
        public ?Element $parentElement;

        public function hasChildNodes(): bool {}

        /**
         * @readonly
         * @var NodeList<Node>
         */
        public NodeList $childNodes;

        /**
         * @readonly
         */
        public ?Node $firstChild;

        /**
         * @readonly
         */
        public ?Node $lastChild;

        /**
         * @readonly
         */
        public ?Node $previousSibling;

        /**
         * @readonly
         */
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

        public function C14N(
            bool $exclusive = false,
            bool $withComments = false,
            ?array $xpath = null,
            ?array $nsPrefixes = null,
        ): string|false {}

        public function C14NFile(
            string $uri,
            bool $exclusive = false,
            bool $withComments = false,
            ?array $xpath = null,
            ?array $nsPrefixes = null,
        ): int|false {}

        public function __sleep(): array {}

        public function __wakeup(): void {}
    }

    /**
     * @template-covariant TNode as Node
     *
     * @template-implements IteratorAggregate<int, TNode>
     */
    class NodeList implements IteratorAggregate, Countable
    {
        /**
         * @readonly
         */
        public int $length;

        public function count(): int {}

        /**
         * @return iterable<int, TNode>
         */
        public function getIterator(): Iterator {}

        /**
         * @return TNode
         */
        public function item(int $index): ?Node {}
    }

    /**
     * @implements IteratorAggregate<array-key, Attr>
     */
    class NamedNodeMap implements IteratorAggregate, Countable
    {
        /**
         * @readonly
         */
        public int $length;

        public function item(int $index): ?Attr {}

        public function getNamedItem(string $qualifiedName): ?Attr {}

        public function getNamedItemNS(?string $namespace, string $localName): ?Attr {}

        public function count(): int {}

        /**
         * @return Iterator<array-key, Attr>
         */
        public function getIterator(): Iterator {}
    }

    /**
     * @implements IteratorAggregate<string, Entity|Notation>
     */
    class DtdNamedNodeMap implements IteratorAggregate, Countable
    {
        /**
         * @readonly
         */
        public int $length;

        public function item(int $index): Entity|Notation|null {}

        public function getNamedItem(string $qualifiedName): Entity|Notation|null {}

        public function getNamedItemNS(?string $namespace, string $localName): Entity|Notation|null {}

        public function count(): int {}

        /**
         * @return Iterator<string, Entity|Notation>
         */
        public function getIterator(): Iterator {}
    }

    /**
     * @implements IteratorAggregate<array-key, Element>
     */
    class HTMLCollection implements IteratorAggregate, Countable
    {
        /**
         * @readonly
         */
        public int $length;

        public function item(int $index): ?Element {}

        public function namedItem(string $key): ?Element {}

        public function count(): int {}

        /**
         * @return Iterator<array-key, Element>
         */
        public function getIterator(): Iterator {}
    }

    enum AdjacentPosition: string
    {
        case BeforeBegin = 'beforebegin';
        case AfterBegin = 'afterbegin';
        case BeforeEnd = 'beforeend';
        case AfterEnd = 'afterend';
    }

    class Element extends Node implements ParentNode, ChildNode
    {
        /**
         * @readonly
         */
        public ?string $namespaceURI;

        /**
         * @readonly
         */
        public ?string $prefix;

        /**
         * @readonly
         */
        public string $localName;

        /**
         * @readonly
         */
        public string $tagName;

        public string $id;
        public string $className;

        /**
         * @readonly
         */
        public TokenList $classList;

        public function hasAttributes(): bool {}

        /**
         * @readonly
         */
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

        /**
         * @readonly
         */
        public ?Element $firstElementChild;

        /**
         * @readonly
         */
        public ?Element $lastElementChild;

        /**
         * @readonly
         */
        public int $childElementCount;

        /**
         * @readonly
         */
        public ?Element $previousElementSibling;

        /**
         * @readonly
         */
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

        public function querySelectorAll(string $selectors): NodeList {}

        public function closest(string $selectors): ?Element {}

        public function matches(string $selectors): bool {}

        public string $innerHTML;

        public string $substitutedNodeValue;

        /** @return list<NamespaceInfo> */
        public function getInScopeNamespaces(): array {}

        /** @return list<NamespaceInfo> */
        public function getDescendantNamespaces(): array {}

        public function rename(?string $namespaceURI, string $qualifiedName): void {}
    }

    class HTMLElement extends Element {}

    class Attr extends Node
    {
        /**
         * @readonly
         */
        public ?string $namespaceURI;

        /**
         * @readonly
         */
        public ?string $prefix;

        /**
         * @readonly
         */
        public string $localName;

        /**
         * @readonly
         */
        public string $name;

        public string $value;

        /**
         * @readonly
         */
        public ?Element $ownerElement;

        /**
         * @readonly
         */
        public bool $specified = true;

        public function isId(): bool {}

        public function rename(?string $namespaceURI, string $qualifiedName): void {}
    }

    class CharacterData extends Node implements ChildNode
    {
        /**
         * @readonly
         */
        public ?Element $previousElementSibling;

        /**
         * @readonly
         */
        public ?Element $nextElementSibling;

        public string $data;

        /**
         * @readonly
         */
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

    class Text extends CharacterData
    {
        public function splitText(int $offset): Text {}

        /**
         * @readonly
         */
        public string $wholeText;
    }

    class CDATASection extends Text {}

    class ProcessingInstruction extends CharacterData
    {
        /**
         * @readonly
         */
        public string $target;
    }

    class Comment extends CharacterData {}

    class DocumentType extends Node implements ChildNode
    {
        /**
         * @readonly
         */
        public string $name;

        /**
         * @readonly
         */
        public DtdNamedNodeMap $entities;

        /**
         * @readonly
         */
        public DtdNamedNodeMap $notations;

        /**
         * @readonly
         */
        public string $publicId;

        /**
         * @readonly
         */
        public string $systemId;

        /**
         * @readonly
         */
        public ?string $internalSubset;

        public function remove(): void {}

        public function before(Node|string ...$nodes): void {}

        public function after(Node|string ...$nodes): void {}

        public function replaceWith(Node|string ...$nodes): void {}
    }

    class DocumentFragment extends Node implements ParentNode
    {
        /**
         * @readonly
         */
        public ?Element $firstElementChild;

        /**
         * @readonly
         */
        public ?Element $lastElementChild;

        /**
         * @readonly
         */
        public int $childElementCount;

        public function appendXml(string $data): bool {}

        public function append(Node|string ...$nodes): void {}

        public function prepend(Node|string ...$nodes): void {}

        public function replaceChildren(Node|string ...$nodes): void {}

        public function querySelector(string $selectors): ?Element {}

        public function querySelectorAll(string $selectors): NodeList {}
    }

    class Entity extends Node
    {
        /**
         * @readonly
         */
        public ?string $publicId;

        /**
         * @readonly
         */
        public ?string $systemId;

        /**
         * @readonly
         */
        public ?string $notationName;
    }

    class EntityReference extends Node {}

    class Notation extends Node
    {
        /**
         * @readonly
         */
        public string $publicId;

        /**
         * @readonly
         */
        public string $systemId;
    }

    abstract class Document extends Node implements ParentNode
    {
        /**
         * @readonly
         */
        public Implementation $implementation;
        public string $URL;
        public string $documentURI;
        public string $characterSet;
        public string $charset;
        public string $inputEncoding;

        /**
         * @readonly
         */
        public ?DocumentType $doctype;

        /**
         * @readonly
         */
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

        /**
         * @readonly
         */
        public ?Element $firstElementChild;

        /**
         * @readonly
         */
        public ?Element $lastElementChild;

        /**
         * @readonly
         */
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

        public function querySelectorAll(string $selectors): NodeList {}

        public ?HTMLElement $body;

        /**
         * @readonly
         */
        public ?HTMLElement $head;
        public string $title;
    }

    final class HTMLDocument extends Document
    {
        public static function createEmpty(string $encoding = 'UTF-8'): HTMLDocument {}

        public static function createFromFile(
            string $path,
            int $options = 0,
            ?string $overrideEncoding = null,
        ): HTMLDocument {}

        public static function createFromString(
            string $source,
            int $options = 0,
            ?string $overrideEncoding = null,
        ): HTMLDocument {}

        public function saveXml(?Node $node = null, int $options = 0): string|false {}

        public function saveXmlFile(string $filename, int $options = 0): int|false {}

        public function saveHtml(?Node $node = null): string {}

        public function saveHtmlFile(string $filename): int|false {}
    }

    final class XMLDocument extends Document
    {
        public static function createEmpty(string $version = '1.0', string $encoding = 'UTF-8'): XMLDocument {}

        public static function createFromFile(
            string $path,
            int $options = 0,
            ?string $overrideEncoding = null,
        ): XMLDocument {}

        public static function createFromString(
            string $source,
            int $options = 0,
            ?string $overrideEncoding = null,
        ): XMLDocument {}

        /**
         * @readonly
         */
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

    final class TokenList implements IteratorAggregate, Countable
    {
        private function __construct() {}

        /**
         * @readonly
         */
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

        public function getIterator(): Iterator {}
    }

    final readonly class NamespaceInfo
    {
        public ?string $prefix;
        public ?string $namespaceURI;
        public Element $element;

        private function __construct() {}
    }

    final class XPath
    {
        /**
         * @readonly
         */
        public Document $document;

        public bool $registerNodeNamespaces;

        public function __construct(Document $document, bool $registerNodeNS = true) {}

        public function evaluate(
            string $expression,
            ?Node $contextNode = null,
            bool $registerNodeNS = true,
        ): null|bool|float|string|NodeList {}

        public function query(string $expression, ?Node $contextNode = null, bool $registerNodeNS = true): NodeList {}

        public function registerNamespace(string $prefix, string $namespace): bool {}

        public function registerPhpFunctions(string|array|null $restrict = null): void {}

        public function registerPhpFunctionNS(string $namespaceURI, string $name, callable $callable): void {}

        public static function quote(string $str): string {}
    }

    function import_simplexml(object $node): Element {}
}
