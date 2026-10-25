<?php

declare(strict_types=1);

/** @param Closure(string $value): string $callback */
function acceptStringClosure(Closure $callback): string
{
    return $callback('value');
}

function acceptString(string $_value): void {}

function stringIdentity(string $value): string
{
    return $value;
}

final class StringTransformer
{
    public static function transformStatic(string $value): string
    {
        return $value;
    }

    public function transform(string $value): string
    {
        return $value;
    }

    public function __invoke(string $value): string
    {
        return $value;
    }
}

function callableForms(StringTransformer $transformer): void
{
    acceptStringClosure(Closure::fromCallable([DOMXPath::class, 'quote']));
    acceptStringClosure(Closure::fromCallable([StringTransformer::class, 'transformStatic']));
    acceptStringClosure(Closure::fromCallable('StringTransformer::transformStatic'));
    acceptStringClosure(Closure::fromCallable([$transformer, 'transform']));
    acceptStringClosure(Closure::fromCallable('stringIdentity'));
    acceptStringClosure(Closure::fromCallable($transformer));
    acceptStringClosure(Closure::fromCallable(fn(string $value): string => $value));
    acceptStringClosure(Closure::fromCallable(stringIdentity(...)));
    acceptStringClosure(Closure::fromCallable(callback: [DOMXPath::class, 'quote']));
}

/** @param callable(string): string $callback */
function typedCallable(callable $callback): void
{
    $closure = Closure::fromCallable($callback);
    acceptStringClosure($closure);
    acceptString($closure('value'));
}

/** @param Closure(string): string $callback */
function typedClosure(Closure $callback): void
{
    $closure = Closure::fromCallable($callback);
    acceptStringClosure($closure);
    acceptString($closure('value'));
}

function optionalString(string $value = ''): string
{
    return $value;
}

function variadicStrings(string ...$values): string
{
    return implode('', $values);
}

function optionalAndVariadicParameters(): void
{
    $optional = Closure::fromCallable('optionalString');
    acceptString($optional());
    acceptString($optional('value'));
    acceptStringClosure($optional);

    $variadic = Closure::fromCallable('variadicStrings');
    acceptString($variadic());
    acceptString($variadic('one', 'two'));
    acceptStringClosure($variadic);

    // @mago-expect analysis:invalid-argument
    $optional(123);
    // @mago-expect analysis:invalid-argument
    $variadic('value', 123);
}

function incompatibleSignatures(): void
{
    // @mago-expect analysis:invalid-argument
    acceptStringClosure(Closure::fromCallable('strlen'));
    // @mago-expect analysis:invalid-argument
    acceptStringClosure(Closure::fromCallable(fn(int $value): string => (string) $value));

    $quote = Closure::fromCallable([DOMXPath::class, 'quote']);
    acceptString($quote('value'));
    // @mago-expect analysis:invalid-argument
    $quote(123);
    // @mago-expect analysis:too-few-arguments
    $quote();
}

function unionSignatures(bool $chooseString): void
{
    $closure = Closure::fromCallable($chooseString ? 'stringIdentity' : 'strlen');
    // @mago-expect analysis:possibly-invalid-argument
    acceptStringClosure($closure);
}

/** @param callable-string $unknown */
function unknownCallableString(bool $chooseKnown, StringTransformer $transformer, string $unknown): void
{
    $closure = Closure::fromCallable($chooseKnown ? [$transformer, 'transform'] : $unknown);
    // @mago-expect analysis:less-specific-nested-argument-type
    acceptStringClosure($closure);
}

function unknownClosure(bool $chooseKnown, StringTransformer $transformer, Closure $unknown): void
{
    $closure = Closure::fromCallable($chooseKnown ? [$transformer, 'transform'] : $unknown);
    // @mago-expect analysis:less-specific-nested-argument-type
    acceptStringClosure($closure);
}

/**
 * @template T
 * @param T $value
 * @return T
 */
function genericIdentity(mixed $value): mixed
{
    return $value;
}

function genericCallable(string $value): string
{
    $identity = Closure::fromCallable('genericIdentity');
    return $identity($value);
}

function mutateString(string &$value): void
{
    $value = strtoupper($value);
}

function byReferenceParameter(string $value): void
{
    $mutate = Closure::fromCallable('mutateString');
    $mutate($value);
    // @mago-expect analysis:invalid-pass-by-reference
    $mutate('value');
}
