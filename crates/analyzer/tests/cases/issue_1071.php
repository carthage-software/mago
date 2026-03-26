<?php

declare(strict_types=1);

class Test
{
    public const CONTENT_TYPE_HTML_FULL = 'html_full';

    public const MIME_TYPES = [
        self::CONTENT_TYPE_HTML_FULL => 'text/html+full',
    ];

    public const CORRECTLY_INFERRED_MIME_TYPES = [
        'html_full' => 'text/html+full',
    ];

    /**
     * @return array<array-key, 'text/html+full'>
     */
    public function getArray(): array
    {
        return self::MIME_TYPES;
    }

    /**
     * @return array{'html_full': 'text/html+full'}
     */
    public function getArray2(): array
    {
        return self::CORRECTLY_INFERRED_MIME_TYPES;
    }
}
