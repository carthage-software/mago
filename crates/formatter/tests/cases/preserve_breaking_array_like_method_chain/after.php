<?php

// When a method chain argument contains a preserved array, the chain should still expand
// if the total width exceeds print-width.
// The same expression written on a single line should produce the same chain expansion.

// Multi-line array in method chain
$html .= $this
    ->msg('parentheses')
    ->params(Message::listParam([
        Message::rawParam($firstLink),
        Message::rawParam($lastLink),
    ], ListType::PIPE))
    ->escaped() . ' ';

// Same expression, single-line array
$html .= $this
    ->msg('parentheses')
    ->params(Message::listParam([Message::rawParam($firstLink), Message::rawParam($lastLink)], ListType::PIPE))
    ->escaped() . ' ';
