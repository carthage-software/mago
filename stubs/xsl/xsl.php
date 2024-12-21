<?php

// Start of xsl v.0.1
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\TentativeType;

/**
 * @link https://php.net/manual/en/class.xsltprocessor.php
 */
class XSLTProcessor
{
    /**
     * Import stylesheet
     * @link https://php.net/manual/en/xsltprocessor.importstylesheet.php
     * @param object $stylesheet <p>
     * The imported style sheet as a <b>DOMDocument</b> or
     * <b>SimpleXMLElement</b> object.
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    #[TentativeType]
    public function importStylesheet(#[LanguageLevelTypeAware(['8.0' => 'object'], default: '')] $stylesheet): bool {}

    /**
     * Transform to a DOMDocument
     * @link https://php.net/manual/en/xsltprocessor.transformtodoc.php
     * @param object $document The DOMDocument or SimpleXMLElement or libxml-compatible object to be transformed.
     * @param string|null $returnClass
     * @return DOMDocument|false The resulting <b>DOMDocument</b> or <b>FALSE</b> on error.
     */
    #[TentativeType]
    public function transformToDoc(
        #[LanguageLevelTypeAware(['8.0' => 'object'], default: '')] $document,
        #[LanguageLevelTypeAware(['8.0' => 'string|null'], default: '')] $returnClass = null
    ): object|false {}

    /**
     * Transform to URI
     * @link https://php.net/manual/en/xsltprocessor.transformtouri.php
     * @param DOMDocument|SimpleXMLElement $document <p>
     * The document to transform.
     * </p>
     * @param string $uri <p>
     * The target URI for the transformation.
     * </p>
     * @return int the number of bytes written or <b>FALSE</b> if an error occurred.
     */
    #[TentativeType]
    public function transformToUri(
        #[LanguageLevelTypeAware(['8.0' => 'object'], default: '')] $document,
        #[LanguageLevelTypeAware(['8.0' => 'string'], default: '')] $uri
    ): int {}

    /**
     * Transform to XML
     * @link https://php.net/manual/en/xsltprocessor.transformtoxml.php
     * @param DOMDocument|SimpleXMLElement $document <p>
     * The transformed document.
     * </p>
     * @return string|false|null The result of the transformation as a string or <b>FALSE</b> on error.
     */
    #[TentativeType]
    public function transformToXml(#[LanguageLevelTypeAware(['8.0' => 'object'], default: '')] $document): string|false|null {}

    /**
     * Set value for a parameter
     * @link https://php.net/manual/en/xsltprocessor.setparameter.php
     * @param string $namespace <p>
     * The namespace URI of the XSLT parameter.
     * </p>
     * @param string $name <p>
     * The local name of the XSLT parameter.
     * </p>
     * @param string $value <p>
     * The new value of the XSLT parameter.
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    #[TentativeType]
    public function setParameter(
        #[LanguageLevelTypeAware(['8.0' => 'string'], default: '')] $namespace,
        #[LanguageLevelTypeAware(['8.0' => 'array|string'], default: '')] $name,
        #[LanguageLevelTypeAware(['8.0' => 'string|null'], default: '')] $value = null
    ): bool {}

    /**
     * Get value of a parameter
     * @link https://php.net/manual/en/xsltprocessor.getparameter.php
     * @param string $namespace <p>
     * The namespace URI of the XSLT parameter.
     * </p>
     * @param string $name <p>
     * The local name of the XSLT parameter.
     * </p>
     * @return string|false The value of the parameter (as a string), or <b>FALSE</b> if it's not set.
     */
    #[TentativeType]
    public function getParameter(
        #[LanguageLevelTypeAware(['8.0' => 'string'], default: '')] $namespace,
        #[LanguageLevelTypeAware(['8.0' => 'string'], default: '')] $name
    ): string|false {}

    /**
     * Remove parameter
     * @link https://php.net/manual/en/xsltprocessor.removeparameter.php
     * @param string $namespace <p>
     * The namespace URI of the XSLT parameter.
     * </p>
     * @param string $name <p>
     * The local name of the XSLT parameter.
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    #[TentativeType]
    public function removeParameter(
        #[LanguageLevelTypeAware(['8.0' => 'string'], default: '')] $namespace,
        #[LanguageLevelTypeAware(['8.0' => 'string'], default: '')] $name
    ): bool {}

    /**
     * Determine if PHP has EXSLT support
     * @link https://php.net/manual/en/xsltprocessor.hasexsltsupport.php
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     * @since 5.0.4
     */
    #[TentativeType]
    public function hasExsltSupport(): bool {}

    /**
     * Enables the ability to use PHP functions as XSLT functions
     * @link https://php.net/manual/en/xsltprocessor.registerphpfunctions.php
     * @param array|string|null $functions [optional] <p>
     * Use this parameter to only allow certain functions to be called from
     * XSLT.
     * </p>
     * <p>
     * This parameter can be either a string (a function name) or an array of
     * functions.
     * </p>
     * @return void No value is returned.
     * @since 5.0.4
     */
    #[TentativeType]
    public function registerPHPFunctions(#[LanguageLevelTypeAware(['8.0' => 'array|string|null'], default: '')] $functions = null): void {}

    /**
     * Sets profiling output file
     * @link https://php.net/manual/en/xsltprocessor.setprofiling.php
     * @param string $filename <p>
     * Path to the file to dump profiling information.
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    public function setProfiling(#[LanguageLevelTypeAware(['8.0' => 'string|null'], default: '')] $filename) {}

    /**
     * Set security preferences
     * @link https://php.net/manual/en/xsltprocessor.setsecurityprefs.php
     * @param int $preferences
     * @return int
     * @since 5.4
     */
    #[TentativeType]
    public function setSecurityPrefs(#[LanguageLevelTypeAware(['8.0' => 'int'], default: '')] $preferences): int {}

    /**
     * Get security preferences
     * @link https://php.net/manual/en/xsltprocessor.getsecurityprefs.php
     * @return int
     * @since 5.4
     */
    #[TentativeType]
    public function getSecurityPrefs(): int {}
}
define('XSL_CLONE_AUTO', 0);
define('XSL_CLONE_NEVER', -1);
define('XSL_CLONE_ALWAYS', 1);

/** @link https://php.net/manual/en/xsl.constants.php */
define('XSL_SECPREF_NONE', 0);
/** @link https://php.net/manual/en/xsl.constants.php */
define('XSL_SECPREF_READ_FILE', 2);
/** @link https://php.net/manual/en/xsl.constants.php */
define('XSL_SECPREF_WRITE_FILE', 4);
/** @link https://php.net/manual/en/xsl.constants.php */
define('XSL_SECPREF_CREATE_DIRECTORY', 8);
/** @link https://php.net/manual/en/xsl.constants.php */
define('XSL_SECPREF_READ_NETWORK', 16);
/** @link https://php.net/manual/en/xsl.constants.php */
define('XSL_SECPREF_WRITE_NETWORK', 32);
/** @link https://php.net/manual/en/xsl.constants.php */
define('XSL_SECPREF_DEFAULT', 44);

/**
 * libxslt version like 10117. Available as of PHP 5.1.2.
 * @link https://php.net/manual/en/xsl.constants.php
 */
define('LIBXSLT_VERSION', 10128);

/**
 * libxslt version like 1.1.17. Available as of PHP 5.1.2.
 * @link https://php.net/manual/en/xsl.constants.php
 */
define('LIBXSLT_DOTTED_VERSION', "1.1.28");

/**
 * libexslt version like 813. Available as of PHP 5.1.2.
 * @link https://php.net/manual/en/xsl.constants.php
 */
define('LIBEXSLT_VERSION', 817);

/**
 * libexslt version like 1.1.17. Available as of PHP 5.1.2.
 * @link https://php.net/manual/en/xsl.constants.php
 */
define('LIBEXSLT_DOTTED_VERSION', "1.1.28");

// End of xsl v.0.1
