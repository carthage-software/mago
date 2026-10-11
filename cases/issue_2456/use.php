<?php

declare(strict_types=1);

function name(): mixed
{
    return fetch()->name;
}

function default_object(): stdClass
{
    return fetch();
}

function defined_object(): stdClass
{
    return fetch_defined();
}

/** @return array<string, mixed> */
function default_array(): array
{
    return fetch_array();
}

function explicit_object(): stdClass
{
    return fetch(MODE_OBJECT);
}

/** @return array<string, mixed> */
function explicit_array(): array
{
    return fetch(MODE_ARRAY);
}

function named_default_object(): stdClass
{
    return fetch_named(unused: 1);
}

/** @return array<string, mixed> */
function named_explicit_array(): array
{
    return fetch_named(mode: MODE_ARRAY);
}

function default_method_object(): stdClass
{
    return new Fetcher()->fetch();
}

function generic_default_object(): stdClass
{
    return fetch_generic();
}

function imported_default_object(): stdClass
{
    return Imported\fetch();
}

function qualified_default_object(): stdClass
{
    return Qualified\fetch();
}
