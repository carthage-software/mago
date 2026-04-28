<?php

declare(strict_types=1);

final class ClassesHookGetSet
{
    private string $backing = '';

    public string $value {
        get => $this->backing;
        set => $this->backing = $value;
    }
}

$obj = new ClassesHookGetSet();
$obj->value = 'hi';
echo $obj->value;
