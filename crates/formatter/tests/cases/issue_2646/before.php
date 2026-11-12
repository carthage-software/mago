<?php

function test1(LongClassName $longParameterName = new LongClassName([new VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName()])) {}

function test2(array $short = [
    1,
    2,
]) {}

function longString(string $veryLongParameterNameThatGoesOnAndOnAndOnAndOnAndOnAndOn = 'some very long default string value that goes past the limit'): int
{
    return 1;
}

function longHint(VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryVeryVeryLongClassName|null $parameter): void
{
    echo 1;
}

function longReturn(LongClassName $longParameterName): VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryLongClassName
{
    return $longParameterName;
}

function byReference(VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryVeryVeryLongClassName &$parameter): void {}

function variadic(VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryVeryVeryLongClassName ...$parameters): void {}

function atTheLimit(string $parameter = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'): void {}

function overTheLimit(string $parameter = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'): void {}

function short(int $a): void
{
    echo $a;
}

function huggedArray(array $options = ['aaaaaaaaaaaaaaaa' => 1111111111111111, 'bbbbbbbbbbbbbbbbbbbbbb' => 222222222222222222, 'ccccccccccccccccccc' => 33333333333333333333]): int
{
    return 1;
}

function huggedNew(LongClassName $longParameterName = new LongClassName(new VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName(), 12345)): void {}

abstract class Example
{
    abstract public function declared(VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryLongClassName $parameter): void;

    public function method(VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryLongClassName $parameter): void
    {
        echo 1;
    }

    public function fits(LongClassName $longParameterName = new LongClassName([new ShortClassName()])): void
    {
        echo 1;
    }
}

interface Contract
{
    public function declared(VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryVeryLongClassName $parameter): void;
}

foo(function (VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryVeryVeryLongClassName|null $parameter) {
    return $parameter;
});

foo($first, function (VeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryVeryLongClassName|null $parameter): void {
    echo $parameter;
});

foo(function (string $value): string {
    return $value;
});

$mapped = array_map(fn(VeryVeryVeryVeryVeryVeryVeryVeryVeryVeryLongClassName|AnotherVeryVeryVeryVeryVeryVeryVeryLongClassName $parameter) => $parameter, $items);

$dateTimeType = Type\converted(Type\string(), Type\instance_of(DateTimeImmutable::class), static function (string $value): DateTimeImmutable {
    return new DateTimeImmutable($value);
});
