<?php

/**
 * @template TModel of object
 *
 * @param class-string<TModel>|string|TModel $_model
 */
function process_model(object|string $_model): void
{
}

process_model('table_name');
process_model(stdClass::class);
process_model(new stdClass());
