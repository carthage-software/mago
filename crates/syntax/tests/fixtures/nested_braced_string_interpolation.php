<?php

$quoted = "{$values["{$keys['}']}"]}";
$commented = "{$values[/* } */ 'key']}";
$shell = `{$values["{$keys['}']}"]}`;
$document = <<<TEXT
{$values["{$keys['}']}"]}
TEXT;

$property = "{$object->{"{$keys['}']}"}}";
$variable = "${"{$keys['}']}"}";
