<?php

final class XMLParser {}

/**
 * @pure
 */
function xml_parser_create(?string $encoding): XMLParser {}

/**
 * @pure
 */
function xml_parser_create_ns(?string $encoding, string $separator = ':'): XMLParser {}

/**
 * @deprecated
 */
function xml_set_object(XMLParser $parser, object $object): bool {}

function xml_set_element_handler(
    XMLParser $parser,
    callable|string|null $start_handler,
    callable|string|null $end_handler,
): bool {}

function xml_set_character_data_handler(XMLParser $parser, callable|string|null $handler): bool {}

function xml_set_processing_instruction_handler(XMLParser $parser, callable|string|null $handler): bool {}

function xml_set_default_handler(XMLParser $parser, callable|string|null $handler): bool {}

function xml_set_unparsed_entity_decl_handler(XMLParser $parser, callable|string|null $handler): bool {}

function xml_set_notation_decl_handler(XMLParser $parser, callable|string|null $handler): bool {}

function xml_set_external_entity_ref_handler(XMLParser $parser, callable|string|null $handler): bool {}

function xml_set_start_namespace_decl_handler(XMLParser $parser, callable|string|null $handler): bool {}

function xml_set_end_namespace_decl_handler(XMLParser $parser, callable|string|null $handler): bool {}

function xml_parse(XMLParser $parser, string $data, bool $is_final = false): int {}

function xml_parse_into_struct(XMLParser $parser, string $data, array &$values, array &$index): int {}

/**
 * @pure
 */
function xml_get_error_code(XMLParser $parser): int {}

/**
 * @pure
 */
function xml_error_string(int $error_code): ?string {}

/**
 * @pure
 */
function xml_get_current_line_number(XMLParser $parser): int {}

/**
 * @pure
 */
function xml_get_current_column_number(XMLParser $parser): int {}

/**
 * @pure
 */
function xml_get_current_byte_index(XMLParser $parser): int {}

#[Deprecated(since: '8.5', message: 'as it has no effect since PHP 8.0')]
function xml_parser_free(XMLParser $parser): bool {}

function xml_parser_set_option(XMLParser $parser, int $option, mixed $value): bool {}

/**
 * @pure
 */
function xml_parser_get_option(XMLParser $parser, int $option): string|int|bool {}

const XML_ERROR_NONE = 0;

const XML_ERROR_NO_MEMORY = 1;

const XML_ERROR_SYNTAX = 2;

const XML_ERROR_NO_ELEMENTS = 3;

const XML_ERROR_INVALID_TOKEN = 4;

const XML_ERROR_UNCLOSED_TOKEN = 5;

const XML_ERROR_PARTIAL_CHAR = 6;

const XML_ERROR_TAG_MISMATCH = 7;

const XML_ERROR_DUPLICATE_ATTRIBUTE = 8;

const XML_ERROR_JUNK_AFTER_DOC_ELEMENT = 9;

const XML_ERROR_PARAM_ENTITY_REF = 10;

const XML_ERROR_UNDEFINED_ENTITY = 11;

const XML_ERROR_RECURSIVE_ENTITY_REF = 12;

const XML_ERROR_ASYNC_ENTITY = 13;

const XML_ERROR_BAD_CHAR_REF = 14;

const XML_ERROR_BINARY_ENTITY_REF = 15;

const XML_ERROR_ATTRIBUTE_EXTERNAL_ENTITY_REF = 16;

const XML_ERROR_MISPLACED_XML_PI = 17;

const XML_ERROR_UNKNOWN_ENCODING = 18;

const XML_ERROR_INCORRECT_ENCODING = 19;

const XML_ERROR_UNCLOSED_CDATA_SECTION = 20;

const XML_ERROR_EXTERNAL_ENTITY_HANDLING = 21;

const XML_OPTION_CASE_FOLDING = 1;

const XML_OPTION_TARGET_ENCODING = 2;

const XML_OPTION_SKIP_TAGSTART = 3;

const XML_OPTION_SKIP_WHITE = 4;

const XML_OPTION_PARSE_HUGE = 5;

const XML_SAX_IMPL = 'libxml';
