<?php

function encodeOrThrowWithConstants(mixed $input): string {
    return json_encode($input, JSON_THROW_ON_ERROR | JSON_PRETTY_PRINT);
}

function encodeOrThrowWithVariable(mixed $input, int $flags): string {
    return json_encode($input, JSON_THROW_ON_ERROR | $flags);
}

function encodeOrThrowWithVariableFirst(mixed $input, int $flags): string {
    return json_encode($input, $flags | JSON_THROW_ON_ERROR);
}

function encodeOrThrowWithNumericLiteral(mixed $input, int $flags): string {
    return json_encode($input, $flags | 4194304);
}

function encodeWithUnknownFlags(mixed $input, int $flags): string|false {
    return json_encode($input, $flags);
}
