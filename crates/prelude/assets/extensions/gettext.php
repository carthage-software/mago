<?php

function textdomain(?string $domain = null): string {}

/**
 * @pure
 */
function _(string $message): string {}

/**
 * @pure
 */
function gettext(string $message): string {}

function dgettext(string $domain, string $message): string {}

function dcgettext(string $domain, string $message, int $category): string {}

function bindtextdomain(string $domain, ?string $directory = null): string|false {}

/**
 * @pure
 */
function ngettext(string $singular, string $plural, int $count): string {}

/**
 * @pure
 */
function dngettext(string $domain, string $singular, string $plural, int $count): string {}

/**
 * @pure
 */
function dcngettext(string $domain, string $singular, string $plural, int $count, int $category): string {}

function bind_textdomain_codeset(string $domain, ?string $codeset = null): string|false {}
