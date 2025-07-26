<?php

class LibXMLError
{
    public int $level;
    public int $code;
    public int $column;
    public string $message;
    public string $file;
    public int $line;
}

/**
 * @param resource $context
 */
function libxml_set_streams_context($context): void
{
}

function libxml_use_internal_errors(null|bool $use_errors = null): bool
{
}

function libxml_get_last_error(): LibXMLError|false
{
}

function libxml_clear_errors(): void
{
}

/**
 * @return list<LibXMLError>
 */
function libxml_get_errors(): array
{
}

/**
 * @deprecated
 */
function libxml_disable_entity_loader(bool $disable = true): bool
{
}

function libxml_set_external_entity_loader(null|callable $resolver_function): bool
{
}

function libxml_get_external_entity_loader(): null|callable
{
}

const LIBXML_VERSION = 20901;

const LIBXML_DOTTED_VERSION = '2.9.1';

const LIBXML_LOADED_VERSION = 20901;

const LIBXML_NOENT = 2;

const LIBXML_DTDLOAD = 4;

const LIBXML_DTDATTR = 8;

const LIBXML_DTDVALID = 16;

const LIBXML_NOERROR = 32;

const LIBXML_NOWARNING = 64;

const LIBXML_NOBLANKS = 256;

const LIBXML_XINCLUDE = 1024;

const LIBXML_NSCLEAN = 8192;

const LIBXML_NOCDATA = 16384;

const LIBXML_NONET = 2048;

const LIBXML_PEDANTIC = 128;

const LIBXML_COMPACT = 65536;

const LIBXML_BIGLINES = 65535;

const LIBXML_NOXMLDECL = 2;

const LIBXML_PARSEHUGE = 524288;

const LIBXML_NOEMPTYTAG = 4;

const LIBXML_SCHEMA_CREATE = 1;

const LIBXML_HTML_NOIMPLIED = 8192;

const LIBXML_HTML_NODEFDTD = 4;

const LIBXML_ERR_NONE = 0;

const LIBXML_ERR_WARNING = 1;

const LIBXML_ERR_ERROR = 2;

const LIBXML_ERR_FATAL = 3;

const LIBXML_RECOVER = 1;

const LIBXML_NO_XXE = 8388608;
