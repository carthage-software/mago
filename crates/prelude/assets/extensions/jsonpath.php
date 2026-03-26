<?php

namespace JsonPath;

use RuntimeException;

class JsonPathException extends \RuntimeException {}

class JsonPath
{
    /**
     * @throws JsonPathException
     */
    public function find(array $data, string $expression): array|false {}
}
