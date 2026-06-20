<?php

declare(strict_types=1);

namespace Acme;

final readonly class Str
{
    /**
     * @return list<non-empty-string>
     */
    public static function split(string $text, string $delimiter = ','): array
    {
        $values = array_map('trim', explode($delimiter, $text));
        $values = array_filter($values);

        return array_values($values);
    }
}
