<?php

declare(strict_types=1);

namespace Mago\Tests\TwigFormatter;

use Twig\Environment;
use Twig\Source;
use Twig\Token;

/** @return list<array{int, mixed}> */
function tokens(Environment $twig, string $source): array
{
    $stream = $twig->tokenize(new Source($source, 'case.twig'));
    $tokens = [];
    while (!$stream->isEOF()) {
        $token = $stream->next();
        if ($token->getType() === Token::OPERATOR_TYPE && in_array($token->getValue(), ['?:', '? :'], true)) {
            $tokens[] = [Token::OPERATOR_TYPE, '?'];
            $tokens[] = [Token::PUNCTUATION_TYPE, ':'];
        } else {
            $tokens[] = [$token->getType(), $token->getValue()];
        }
    }

    return $tokens;
}
