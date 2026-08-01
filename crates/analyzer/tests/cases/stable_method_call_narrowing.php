<?php

declare(strict_types=1);

enum ColorKind
{
    case Basic;
    case Ansi256;
    case Rgb;
}

readonly class Color
{
    public function __construct(private ColorKind $kind) {}

    /** @mutation-free */
    public function getKind(): ColorKind
    {
        return $this->kind;
    }
}

function formatColor(Color $color): string
{
    return match ($color->getKind()) {
        ColorKind::Basic => 'basic',
        ColorKind::Ansi256 => 'ansi256',
        ColorKind::Rgb => 'rgb',
    };
}

enum TokenKind
{
    case Identifier;
    case Plus;
}

readonly class Token
{
    public function __construct(public TokenKind $kind) {}
}

final class TokenStream
{
    private int $cursor = 0;

    /** @mutation-free */
    public function peek(): Token
    {
        return new Token($this->cursor === 0 ? TokenKind::Identifier : TokenKind::Plus);
    }

    public function consume(): void
    {
        $this->cursor++;
    }
}

function inspectTokenStream(TokenStream $stream): void
{
    if ($stream->peek()->kind !== TokenKind::Identifier) {
        return;
    }

    $stream->consume();

    if ($stream->peek()->kind === TokenKind::Plus) {
        echo 'plus';
    }
}

final class MutableInput
{
    private int $cursor = 0;

    /** @mutation-free */
    public function hasReachedEnd(): bool
    {
        return $this->cursor >= 2;
    }

    public function read(): void
    {
        $this->cursor++;
    }
}

final readonly class Lexer
{
    public function __construct(private MutableInput $input) {}

    /** @mutation-free */
    public function hasReachedEnd(): bool
    {
        return $this->input->hasReachedEnd();
    }

    public function advance(): void
    {
        if ($this->hasReachedEnd()) {
            return;
        }

        $this->input->read();

        if (!$this->hasReachedEnd()) {
            echo 'more input';
        }
    }
}
