<?php

declare(strict_types=1);

namespace Imported {
    use stdClass;

    use const MODE_OBJECT as OBJECT_MODE;

    /**
     * @param 'object'|'array' $mode
     * @return ($mode is 'array' ? array<string, mixed> : stdClass)
     */
    function fetch(string $mode = OBJECT_MODE): array|stdClass
    {
        return $mode === 'array' ? [] : new stdClass();
    }
}

namespace Qualified {
    use stdClass;

    /**
     * @param 'object'|'array' $mode
     * @return ($mode is 'array' ? array<string, mixed> : stdClass)
     */
    function fetch(string $mode = \MODE_OBJECT): array|stdClass
    {
        return $mode === 'array' ? [] : new stdClass();
    }
}
