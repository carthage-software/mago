<?php

declare(strict_types=1);

/**
 * @return false|array{rdir:string,name:string,read:int,write:int}
 */
function vdir_find(): array|false
{
    if (mt_rand(0, max: 1)) {
        return ['rdir' => 'r', 'name' => 'some', 'read' => 0, 'write' => 713];
    }

    return false;
}

/**
 * @param non-empty-string $s
 */
function i_take_nes(string $s): void
{
    echo $s;
}

function test(): void
{
    $vd = vdir_find();
    if (!empty($vd['rdir'])) {
        i_take_nes($vd['rdir']);
        $pi = $vd['rdir'] . 't.jpg';
        echo $pi;
    }
}
