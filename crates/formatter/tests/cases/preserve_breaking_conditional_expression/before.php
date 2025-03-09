<?php

$options = $this->multiple
    ? ($this->search)($query)
    : [self::CANCEL, self::SEARCH_AGAIN, ...($this->search)($query)];

    $this->now = $now instanceof DateTimeInterface
        ? DateTimeImmutable::createFromInterface($now)
        : new DateTimeImmutable($now);

         function getControls(): array
        {
            return [
                ...($this->bufferEnabled ? [
                    'esc' => 'select',
                ] : [
                    '/' => 'filter',
                    'space' => 'select',
                ]),
                '↑' => 'up',
                '↓' => 'down',
                'enter' => $this->options->getSelectedOptions() === []
                    ? 'skip'
                    : 'confirm',
                'ctrl+c' => 'cancel',
            ];
        }
