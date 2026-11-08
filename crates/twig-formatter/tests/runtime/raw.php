<?php

declare(strict_types=1);

namespace Mago\Tests\TwigFormatter;

use Twig\Node\Node;
use Twig\Token;
use Twig\TokenParser\AbstractTokenParser;

final class RawTokenParser extends AbstractTokenParser
{
    public function parse(Token $token): Node
    {
        $this->parser->getStream()->expect(Token::BLOCK_END_TYPE);
        $body = $this->parser->subparse(static fn(Token $token): bool => $token->test('endraw'), true);
        $this->parser->getStream()->expect(Token::BLOCK_END_TYPE);
        return $body;
    }

    public function getTag(): string
    {
        return 'raw';
    }
}
