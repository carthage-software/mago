<?php

declare(strict_types=1);

final class SplitUnpackFoo {}

final class SplitUnpackBar {}

/**
 * @type SplitUnpackObjects = array{foo: SplitUnpackFoo, bar: SplitUnpackBar}
 */
interface ParameterVariableSplitUnpacking
{
    /** @param SplitUnpackObjects[$key] $value */
    public function store(string $key, object $value): void;
}

function exercise_parameter_variable_split_unpacking(ParameterVariableSplitUnpacking $types): void
{
    $types->store(...['value' => new SplitUnpackFoo()], ...['key' => 'foo']);

    // @mago-expect analysis:invalid-argument
    $types->store(...['value' => new SplitUnpackBar()], ...['key' => 'foo']);
}
