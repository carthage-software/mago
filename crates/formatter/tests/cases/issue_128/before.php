<?php

$permission = Permission::query()->whereField('name', $name)->first();

$this->renderer = new ChoiceRenderer(multiple: $multiple, default: (string) $default);

$this->writeln(implode(PHP_EOL, $complete->complete(
    $command,
    $argumentBag,
    $current,
)));

return new ImmutableString($string)
    ->truncate($this->maxLineCharacters - 1 - $maxLineOffset, end: '…') // -1 is for the ellipsis
    ->toString();
