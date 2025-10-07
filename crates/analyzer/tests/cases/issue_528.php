<?php

/**
 * @param array{'required_field': string, ...} $value
 */
function i_take_flexible_type(array $value): void
{
    echo 'Received array';
}

i_take_flexible_type(['required_field' => 'yes']);
i_take_flexible_type(['required_field' => 'yes', 'optional_field' => 'yes']);
i_take_flexible_type(['foo' => 'bar']); // @mago-expect analysis:possibly-invalid-argument
