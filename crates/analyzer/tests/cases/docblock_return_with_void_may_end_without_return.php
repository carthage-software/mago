<?php

declare(strict_types=1);

final class DocblockVoidResult
{
}

/**
 * @return DocblockVoidResult|void
 */
function docblock_return_with_void_may_end_without_return(bool $condition)
{
    if ($condition) {
        return new DocblockVoidResult();
    }
}

/**
 * @return string|void
 */
function docblock_string_or_void_may_end_without_return(bool $condition)
{
    if ($condition) {
        return 'value';
    }
}
