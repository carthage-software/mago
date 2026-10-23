<?php

declare(strict_types=1);

namespace Issue2548;

final class User
{
    public function getName(): string
    {
        return 'Ada';
    }
}

function staticArrow(): string
{
    return new User() |> (static fn($user) => $user->getName());
}

function arrow(User $user): string
{
    return $user |> (fn($value) => $value->getName());
}

function closure(User $user): string
{
    return $user
        |> function ($value) {
            return $value->getName();
        };
}

function staticClosure(User $user): string
{
    return $user
        |> static function ($value) {
            return $value->getName();
        };
}

function parenthesized(User $user): string
{
    // @mago-format-ignore-next
    return $user |> ((static fn($value) => $value->getName()));
}

function chained(User $user): int
{
    return $user |> (static fn($value) => $value->getName()) |> (static fn($name) => strlen($name));
}

function shapedArray(User $user): string
{
    return ['user' => $user] |> (static fn($value) => $value['user']->getName());
}

function nullableInput(?User $user): string
{
    return $user |> (static fn($value) => $value === null ? 'anonymous' : $value->getName());
}

function unionInput(User|string $user): string
{
    return $user
        |> static function ($value): string {
            if (is_string($value)) {
                return $value;
            }

            return $value->getName();
        };
}

function variadicInput(User $user): string
{
    return $user
        |> static function (...$values): string {
            foreach ($values as $value) {
                return $value->getName();
            }

            return 'anonymous';
        };
}

function capturedInput(User $user, string $prefix): string
{
    return $user
        |> static function ($value) use ($prefix): string {
            return $prefix . $value->getName();
        };
}

function assignedInput(): string
{
    return ($created = new User()) |> (static fn($value) => $created->getName() . $value->getName());
}

function nestedPipe(User $user): int
{
    return $user |> (static fn($value) => $value->getName() |> (static fn($name) => strlen($name)));
}

function explicitParameterType(User $user): string
{
    // @mago-expect analysis:invalid-argument
    return $user |> (static fn(string $value): string => $value);
}

function nonReferenceableInput(): string
{
    // @mago-expect analysis:invalid-pass-by-reference
    return new User()
        |> static function (&$value) {
            return $value->getName();
        };
}

function explicitMixedParameter(User $user): string
{
    return $user |> (static fn(mixed $value) => $value->getName());
}

function explicitArrayParameter(User $user): string
{
    return ['user' => $user] |> (static fn(array $value) => $value['user']->getName());
}

function documentedParameterType(User $user): string
{
    // @mago-expect analysis:invalid-argument
    return $user |> (/** @param string $value */ static fn($value): string => $value);
}

function nestedClosure(User $user): \Closure
{
    // The pipe input belongs to the outer closure's parameter only.
    // @mago-expect analysis:mixed-method-access
    return $user |> (static fn($value) => static fn($other) => $other->getName());
}

function storedCallable(User $user): mixed
{
    // @mago-expect analysis:mixed-method-access
    $callback = static fn($value) => $value->getName();

    return $user |> $callback;
}

function callableFactory(\Closure $callback): \Closure
{
    return $callback;
}

function closureInsideCallableFactory(User $user): mixed
{
    // @mago-expect analysis:mixed-method-access
    return $user |> callableFactory(static fn($value) => $value->getName());
}

/** @param \Closure(User): string $callback */
function invokeUserCallback(\Closure $callback): string
{
    return $callback(new User());
}

function closureInput(): string
{
    return static function ($user): string {
        return $user->getName();
    }
        |> invokeUserCallback(...);
}

function closureInputWithDeclaredCallback(): string
{
    return static function ($user): string {
        return $user->getName();
    }
        |> (
            /** @param \Closure(User): string $callback */ static fn(\Closure $callback): string => $callback(
                new User(),
            )
        );
}

function closureInputWithBoundThis(): string
{
    return function (): string {
        return $this->getName();
    }
        |> (/** @param-closure-this User $callback */ static fn($callback): string => 'bound');
}

/** @param 1 $value */
function takesOne(int $value): void {}

/** @return 0 */
function inputIsAnalyzedOnce(): int
{
    $counter = 0;
    $previous = $counter++ |> (static fn($value) => $value);

    takesOne($counter);

    return $previous;
}
