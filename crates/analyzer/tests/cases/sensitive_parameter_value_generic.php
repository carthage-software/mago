<?php

declare(strict_types=1);

function takes_string(string $value): void {}

function takes_int(int $value): void {}

$wrapped = new SensitiveParameterValue('secret');
takes_string($wrapped->getValue());
// @mago-expect analysis:invalid-argument
takes_int($wrapped->getValue());

$int_wrapped = new SensitiveParameterValue(1);
takes_int($int_wrapped->getValue());
// @mago-expect analysis:invalid-argument
takes_string($int_wrapped->getValue());

/** @param SensitiveParameterValue<string> $wrapped */
function accepts_string(SensitiveParameterValue $wrapped): void
{
    takes_string($wrapped->getValue());
    // @mago-expect analysis:invalid-argument
    takes_int($wrapped->getValue());
}

/** @param SensitiveParameterValue<int|string> $wrapped */
function accepts_union(SensitiveParameterValue $wrapped): void
{
    // @mago-expect analysis:possibly-invalid-argument
    accepts_string($wrapped);
}

accepts_union(new SensitiveParameterValue(1));

/** @return SensitiveParameterValue<string> */
function returns_wrong_value(): SensitiveParameterValue
{
    // @mago-expect analysis:invalid-return-statement
    return new SensitiveParameterValue(1);
}

/** @param SensitiveParameterValue $bare */
function falls_back_to_mixed(SensitiveParameterValue $bare, SensitiveParameterValue $native, mixed $candidate): void
{
    // @mago-expect analysis:mixed-argument
    takes_string($bare->getValue());
    // @mago-expect analysis:mixed-argument
    takes_string($native->getValue());
    if ($candidate instanceof SensitiveParameterValue) {
        // @mago-expect analysis:mixed-argument
        takes_string($candidate->getValue());
    }
}
