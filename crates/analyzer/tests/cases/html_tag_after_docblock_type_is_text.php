<?php

declare(strict_types=1);

/**
 * @return array
 * <pre>
 * ['a' => ['type' => 'x']]
 * </pre>
 */
function html_tag_after_docblock_type_is_text(): array
{
    return ['a' => ['type' => 'x']];
}

/**
 * @return array<
 *     string,
 *     int
 * >
 */
function generic_arguments_may_span_lines(): array
{
    return ['a' => 1];
}
