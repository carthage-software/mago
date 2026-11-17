<?php

declare(strict_types=1);

/**
 * @param array<string, string> $request
 */
function redirect_for(array $request): string
{
    if (isset($request['type']) && 'navigation' === $request['type'] && !empty($request['id'])) {
        return 'navigation item';
    }

    if (isset($request['type']) && 'navigation' === $request['type'] && empty($request['id'])) {
        return 'navigation';
    }

    if (isset($request['type']) && 'page' === $request['type'] && (empty($request['canvas']) || empty($request['id']))) {
        return 'page';
    }

    if (isset($request['type']) && 'template' === $request['type'] && !empty($request['id'])) {
        return 'template';
    }

    return '';
}
