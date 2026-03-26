<?php

function xmlrpc_encode(mixed $value): string {}

function xmlrpc_decode(string $xml, string $encoding = 'iso-8859-1'): mixed {}

function xmlrpc_decode_request(string $xml, string &$method, string $encoding = null): mixed {}

function xmlrpc_encode_request(string $method, mixed $params, ?array $output_options = null): string {}

/**
 * @return 'base64'|'datetime'
 */
function xmlrpc_get_type(mixed $value): string {}

/**
 * @param 'base64'|'datetime' $type
 */
function xmlrpc_set_type(string &$value, string $type) {}

function xmlrpc_is_fault(array $arg): bool {}

/**
 * @return object|resource
 */
function xmlrpc_server_create(): mixed {}

/**
 * @param object|resource $server
 */
function xmlrpc_server_destroy(mixed $server): int {}

/**
 * @param object|resource $server
 */
function xmlrpc_server_register_method(mixed $server, string $method_name, callable $function): bool {}

/**
 * @param object|resource $server
 */
function xmlrpc_server_call_method(
    mixed $server,
    string $xml,
    mixed $user_data,
    ?array $output_options = null,
): string {}

function xmlrpc_parse_method_descriptions(string $xml): array {}

/**
 * @param object|resource $server
 */
function xmlrpc_server_add_introspection_data(mixed $server, array $desc): int {}

/**
 * @param object|resource $server
 */
function xmlrpc_server_register_introspection_callback(mixed $server, string $function): bool {}
