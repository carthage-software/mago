<?php

declare(strict_types=1);

/**
 * @template T
 */
interface NativeReturnRefined {
    /** @return int */
    public function nullable();
    /** @return list<int> */
    public function list();
    /** @return non-empty-string */
    public function nonEmpty();
    /** @return T */
    public function template();
    /** @return Iterator */
    public function iterator();
    /** @return static */
    public function fluent();
    /** @return callable */
    public function get();
}

/**
 * @implements NativeReturnRefined<bool>
 */
final class NativeReturnRefiner implements NativeReturnRefined
{
    public function nullable(): ?int { return 1; }
    public function list(): array { return [1]; }
    public function nonEmpty(): string { return "a"; }
    public function template(): bool { return false; }
    public function iterator(): Countable { return new ArrayIterator([]); }
    public function fluent(): self { return $this; }
    public function get(): array { return [$this, "run"]; }
    public function run(): void {}
}

/** @template T */
abstract class NativeReturnLexer {
    /** @return T|null */
    abstract protected function getType(string $value);
}

/** @template-extends NativeReturnLexer<NativeReturnDocLexer::T_*> */
final class NativeReturnDocLexer extends NativeReturnLexer {
    public const T_NONE = 1;
    public const T_INTEGER = 2;
    protected function getType(string $value): int { return self::T_NONE; }
}

/**
 * @extends ArrayObject<int, int>
 */
final class NativeReturnArrayObject extends ArrayObject
{
    public function natsort(): bool { return true; }
}

final class NativeReturnStringable
{
    public function __toString(): string
    {
        return 'foo';
    }
}

interface NativeReturnDisjoint {
    /** @return int */
    public function check();
    /** @return string */
    public function a();
    /** @return string */
    public function b();
    /** @return string */
    public function c(): mixed;
}

final class NativeReturnDisjointer implements NativeReturnDisjoint {
    /** @mago-expect analysis:incompatible-return-type */
    public function check(): bool { return false; }
    /** @mago-expect analysis:incompatible-return-type */
    public function a(): NativeReturnStringable { return new NativeReturnStringable(); }
    /** @mago-expect analysis:incompatible-return-type */
    public function b(): Stringable { return new NativeReturnStringable(); }
    /** @mago-expect analysis:incompatible-return-type */
    public function c(): int { return 1; }
}
