<?php

declare(strict_types=1);

/**
 * @psalm-type Token = array{kind: string, value: string}
 */
final class LexerB
{
    /**
     * @return Token
     */
    public function ident(string $v): array
    {
        return ['kind' => 'ident', 'value' => $v];
    }
}

/**
 * @psalm-import-type Token from LexerB
 */
final class ParserB
{
    /**
     * @param Token $t
     */
    public function dump(array $t): string
    {
        return $t['kind'] . ':' . $t['value'];
    }
}

$lex = new LexerB();
$par = new ParserB();
echo $par->dump($lex->ident('foo'));
