<?php

function getControls(): array
{
    return [
        ...($this->bufferEnabled ? ['esc' => 'select'] : ['/' => 'filter', 'space' => 'select']),
        '↑' => 'up',
        '↓' => 'down',
        'ctrl+c' => 'cancel',
    ];
}
