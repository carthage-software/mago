<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

use function array_pop;
use function explode;
use function preg_match;
use function strtolower;

/**
 * Validates names carried by direct codebase definitions.
 *
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 */
final class DefinitionName
{
    private const IDENTIFIER_PATTERN = '/\A[A-Z_a-z\x80-\xff][0-9A-Z_a-z\x80-\xff]*\z/D';
    private const VARIABLE_PATTERN = '/\A\$[A-Z_a-z\x80-\xff][0-9A-Z_a-z\x80-\xff]*\z/D';
    private const SYMBOL_PATTERN = '/\A[A-Z_a-z\x80-\xff][0-9A-Z_a-z\x80-\xff]*(?:\\\\[A-Z_a-z\x80-\xff][0-9A-Z_a-z\x80-\xff]*)*\z/D';

    public static function assertVariable(string $name, string $description): void
    {
        if (preg_match(self::VARIABLE_PATTERN, $name) !== 1) {
            throw new InvalidArgumentException("{$description} must be a valid PHP variable name.");
        }
    }

    public static function assertIdentifier(string $name, string $description): void
    {
        if (preg_match(self::IDENTIFIER_PATTERN, $name) !== 1) {
            throw new InvalidArgumentException("{$description} must be a valid PHP identifier.");
        }
    }

    public static function assertClassConstant(string $name, string $description): void
    {
        self::assertIdentifier($name, $description);

        if (strtolower($name) === 'class') {
            throw new InvalidArgumentException("{$description} cannot use the reserved name `class`.");
        }
    }

    public static function assertSymbol(string $name, string $description): void
    {
        if (preg_match(self::SYMBOL_PATTERN, $name) !== 1) {
            throw new InvalidArgumentException("{$description} must be a valid PHP symbol name.");
        }

        $parts = explode('\\', $name);
        $symbol = array_pop($parts);
        foreach ($parts as $part) {
            if (self::isNamespaceKeyword($part)) {
                throw new InvalidArgumentException("{$description} contains the reserved namespace segment `{$part}`.");
            }
        }

        if (self::isReservedSymbol($symbol)) {
            throw new InvalidArgumentException("{$description} cannot use the reserved name `{$symbol}`.");
        }
    }

    private static function isNamespaceKeyword(string $name): bool
    {
        return match (strtolower($name)) {
            'enum', 'from' => true,
            default => self::isReservedSymbol($name),
        };
    }

    private static function isReservedSymbol(string $name): bool
    {
        return match (strtolower($name)) {
            'static',
            'abstract',
            'final',
            'for',
            'private',
            'protected',
            'public',
            'include',
            'include_once',
            'eval',
            'require',
            'require_once',
            'or',
            'xor',
            'and',
            'instanceof',
            'new',
            'clone',
            'exit',
            'die',
            'if',
            'elseif',
            'else',
            'endif',
            'echo',
            'do',
            'while',
            'endwhile',
            'endfor',
            'foreach',
            'endforeach',
            'declare',
            'enddeclare',
            'as',
            'try',
            'catch',
            'finally',
            'throw',
            'use',
            'insteadof',
            'global',
            'var',
            'unset',
            'isset',
            'empty',
            'continue',
            'goto',
            'function',
            'const',
            'return',
            'print',
            'yield',
            'list',
            'switch',
            'endswitch',
            'case',
            'default',
            'break',
            'array',
            'callable',
            'extends',
            'implements',
            'namespace',
            'trait',
            'interface',
            'class',
            '__class__',
            '__trait__',
            '__function__',
            '__method__',
            '__line__',
            '__file__',
            '__dir__',
            '__namespace__',
            '__property__',
            '__halt_compiler',
            'fn',
            'match',
            'parent',
            'self',
            'true',
            'false',
            'null',
            'readonly',
                => true,
            default => false,
        };
    }
}
