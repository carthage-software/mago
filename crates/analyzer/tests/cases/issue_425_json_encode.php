<?php

function encode(mixed $input): string|false {
    return json_encode($input);
}

function encodeOrThrow(mixed $input): string {
    return json_encode($input, JSON_THROW_ON_ERROR);
}
