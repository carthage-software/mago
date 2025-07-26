<?php

/**
 * @var int
 */
const XSL_CLONE_AUTO = 0;

/**
 * @var int
 */
const XSL_CLONE_NEVER = -1;

/**
 * @var int
 */
const XSL_CLONE_ALWAYS = 1;

/**
 * @var int
 */
const XSL_SECPREF_NONE = UNKNOWN;

/**
 * @var int
 */
const XSL_SECPREF_READ_FILE = UNKNOWN;

/**
 * @var int
 */
const XSL_SECPREF_WRITE_FILE = UNKNOWN;

/**
 * @var int
 */
const XSL_SECPREF_CREATE_DIRECTORY = UNKNOWN;

/**
 * @var int
 */
const XSL_SECPREF_READ_NETWORK = UNKNOWN;

/**
 * @var int
 */
const XSL_SECPREF_WRITE_NETWORK = UNKNOWN;

/**
 * @var int
 */
const XSL_SECPREF_DEFAULT = UNKNOWN;

/**
 * @var int
 */
const LIBXSLT_VERSION = UNKNOWN;

/**
 * @var string
 */
const LIBXSLT_DOTTED_VERSION = UNKNOWN;

/**
 * @var int
 */
const LIBEXSLT_VERSION = UNKNOWN;

/**
 * @var string
 */
const LIBEXSLT_DOTTED_VERSION = UNKNOWN;

class XSLTProcessor
{
    public bool $doXInclude = false;

    public bool $cloneDocument = false;

    public int $maxTemplateDepth;

    public int $maxTemplateVars;

    /**
     * @param DOMDocument|DOM\Document|SimpleXMLElement $stylesheet
     */
    public function importStylesheet(object $stylesheet): bool
    {
    }

    /**
     * @param DOMDocument|DOM\Document|SimpleXMLElement $document
     */
    public function transformToDoc(object $document, null|string $returnClass = null): object|false
    {
    }

    /**
     * @param DOMDocument|DOM\Document|SimpleXMLElement $document
     */
    public function transformToUri(object $document, string $uri): int
    {
    }

    /**
     * @param DOMDocument|DOM\Document|SimpleXMLElement $document
     */
    public function transformToXml(object $document): string|null|false
    {
    }

    public function setParameter(string $namespace, array|string $name, null|string $value = null): bool
    {
    }

    public function getParameter(string $namespace, string $name): string|false
    {
    }

    public function removeParameter(string $namespace, string $name): bool
    {
    }

    public function hasExsltSupport(): bool
    {
    }

    public function registerPHPFunctions(array|string|null $functions = null): void
    {
    }

    public function registerPHPFunctionNS(string $namespaceURI, string $name, callable $callable): void
    {
    }

    public function setProfiling(null|string $filename): true
    {
    }

    public function setSecurityPrefs(int $preferences): int
    {
    }

    public function getSecurityPrefs(): int
    {
    }
}
