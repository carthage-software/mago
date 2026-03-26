<?php

declare(strict_types=1);

$text = '[b]bold[i]+italic[/i][/b][i]italic[/b]';

$parts = explode('[', $text);
$out = [];
$partcount = count($parts);
for ($i = 0; $i < $partcount; ++$i) {
    if (!$i) {
        if ($parts[$i] > '') {
            $out[] = [3, 'text', $parts[$i], []];
        }

        continue;
    }
}
