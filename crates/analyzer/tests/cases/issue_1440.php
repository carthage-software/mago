<?php

declare(strict_types=1);

/**
 * @param array<string, mixed> $params
 * @return array<string, scalar>
 */
function getParamsArray(array $params): array
{
    return array_filter($params, is_scalar(...));
}
