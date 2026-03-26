<?php

namespace Parle;

use Exception;
use Throwable;

class ParserException extends Exception implements Throwable {}

class LexerException extends Exception implements Throwable {}

class Token
{
    /**
     * @var int
     */
    public const EOI = 0;

    /**
     * @var int
     */
    public const UNKNOWN = -1;

    /**
     * @var int
     */
    public const SKIP = -2;

    /**
     * @var int
     */
    public $id;

    /**
     * @var string
     */
    public $value;
}

class ErrorInfo
{
    /**
     * @var int
     */
    public $id;

    /**
     * @var int
     */
    public $position;

    /**
     * @var Token|null
     */
    public $token;
}

class Lexer
{
    public const ICASE = 1;
    public const DOT_NOT_LF = 2;
    public const DOT_NOT_CRLF = 4;
    public const SKIP_WS = 8;
    public const MATCH_ZERO_LEN = 16;

    /**
     * @var bool
     */
    public $bol = false;

    /**
     * @var int
     */
    public $flags = 0;

    /**
     * @var int
     *
     * @readonly
     */
    public $state = 0;

    /**
     * @var int
     *
     * @readonly
     */
    public $marker = 0;

    /**
     * @var int
     *
     * @readonly
     */
    public $cursor = 0;

    public function advance(): void {}

    public function build(): void {}

    /**
     * @param (callable(): void) $callback
     */
    public function callout(int $id, callable $callback): void {}

    public function consume(string $data): void {}

    public function dump(): void {}

    public function getToken(): Token {}

    public function insertMacro(string $name, string $regex): void {}

    public function push(string $regex, int $id): void {}

    public function reset(int $pos): void {}
}

class Parser
{
    public const ACTION_ERROR = 0;
    public const ACTION_SHIFT = 1;
    public const ACTION_REDUCE = 2;
    public const ACTION_GOTO = 3;
    public const ACTION_ACCEPT = 4;
    public const ERROR_SYNTAX = 0;
    public const ERROR_NON_ASSOCIATIVE = 1;
    public const ERROR_UNKNOWN_TOKEN = 2;

    /**
     * @var int
     *
     * @readonly
     */
    public $action = 0;

    /**
     * @var int
     *
     * @readonly
     */
    public $reduceId = 0;

    public function advance(): void {}

    public function build(): void {}

    public function consume(string $data, Lexer $lexer): void {}

    public function dump(): void {}

    public function errorInfo(): ErrorInfo {}

    public function left(string $token): void {}

    public function nonassoc(string $token): void {}

    public function precedence(string $token): void {}

    public function push(string $name, string $rule): int {}

    public function reset(int $tokenId): void {}

    public function right(string $token): void {}

    public function sigil(int $idx): string {}

    public function token(string $token): void {}

    public function tokenId(string $token): int {}

    public function trace(): string {}

    public function validate(string $data, Lexer $lexer): bool {}
}

/**
 * @template TValue
 */
class Stack
{
    /**
     * @var bool
     *
     * @readonly
     */
    public $empty = true;

    /**
     * @var int
     *
     * @readonly
     */
    public $size = 0;

    /**
     * @var TValue
     *
     * @readonly
     */
    public $top;

    public function pop(): void {}

    /**
     * @param TValue $item
     *
     * @return void
     */
    public function push($item) {}
}

class RLexer
{
    public const ICASE = 1;
    public const DOT_NOT_LF = 2;
    public const DOT_NOT_CRLF = 4;
    public const SKIP_WS = 8;
    public const MATCH_ZERO_LEN = 16;

    /**
     * @var bool
     */
    public $bol = false;

    /**
     * @var int
     */
    public $flags = 0;

    /**
     * @var int
     *
     * @readonly
     */
    public $state = 0;

    /**
     * @var int
     *
     * @readonly
     */
    public $marker = 0;

    /**
     * @var int
     *
     * @readonly
     */
    public $cursor = 0;

    public function advance(): void {}

    public function build(): void {}

    /**
     * @param (callable(): void) $callback
     */
    public function callout(int $id, callable $callback): void {}

    public function consume(string $data): void {}

    public function dump(): void {}

    public function getToken(): Token {}

    public function push(string $regex, int $id): void {}

    public function push(string $state, string $regex, int $id, string $newState): void {}

    public function push(string $state, string $regex, string $newState): void {}

    public function pushState(string $state): int {}

    public function reset(int $pos): void {}
}

class RParser
{
    public const ACTION_ERROR = 0;
    public const ACTION_SHIFT = 1;
    public const ACTION_REDUCE = 2;
    public const ACTION_GOTO = 3;
    public const ACTION_ACCEPT = 4;
    public const ERROR_SYNTAX = 0;
    public const ERROR_NON_ASSOCIATIVE = 1;
    public const ERROR_UNKNOWN_TOKEN = 2;

    /**
     * @var int
     *
     * @readonly
     */
    public $action = 0;

    /**
     * @var int
     *
     * @readonly
     */
    public $reduceId = 0;

    public function advance(): void {}

    public function build(): void {}

    public function consume(string $data, Lexer $lexer): void {}

    public function dump(): void {}

    public function errorInfo(): ErrorInfo {}

    public function left(string $token): void {}

    public function nonassoc(string $token): void {}

    public function precedence(string $token): void {}

    public function push(string $name, string $rule): int {}

    public function reset(int $tokenId): void {}

    public function right(string $token): void {}

    public function sigil(int $idx): string {}

    public function token(string $token): void {}

    public function tokenId(string $token): int {}

    public function trace(): string {}

    public function validate(string $data, RLexer $lexer): bool {}
}
