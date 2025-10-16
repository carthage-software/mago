<?php

declare(strict_types=1);

final class TokenType
{
    public const T_NONE = 1;
}

/**
 * @template T of UnitEnum|string|int
 * @template V of string|int
 */
final class Token
{
    /**
     * The string value of the token in the input string
     *
     * @readonly
     * @var V
     */
    public string|int $value;

    /**
     * The type of the token (identifier, numeric, string, input parameter, none)
     *
     * @readonly
     * @var T|null
     */
    public $type;

    /**
     * The position of the token in the input string
     *
     * @readonly
     */
    public int $position;

    /**
     * @param V      $value
     * @param T|null $type
     */
    public function __construct(string|int $value, $type, int $position)
    {
        $this->value = $value;
        $this->type = $type;
        $this->position = $position;
    }

    /** @param T ...$types */
    public function isA(...$types): bool
    {
        return in_array($this->type, $types, true);
    }
}

/**
 * Base class for writing simple lexers, i.e. for creating small DSLs.
 *
 * @template T of UnitEnum|string|int
 * @template V of string|int
 */
abstract class AbstractLexer
{
    /**
     * The next token in the input.
     *
     * @var Token<T, V>|null
     */
    public Token|null $lookahead;

    /**
     * Checks whether a given token matches the current lookahead.
     *
     * @param T $type
     *
     * @return bool
     *
     * @psalm-assert-if-true !=null $this->lookahead
     */
    public function isNextToken(int|string|UnitEnum $type)
    {
        return $this->lookahead !== null && $this->lookahead->isA($type);
    }
}

/**
 * @extends AbstractLexer<TokenType::T_*, string>
 */
class Lexer extends AbstractLexer
{
}

$lexer = new Lexer();
$lexer->isNextToken(TokenType::T_NONE);
