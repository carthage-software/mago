<?php

enum Color: string
{
    // @mago-format-ignore-next
    case   RED   =   '#FF0000';

    // @mago-format-ignore-next
    case   GREEN   =   '#00FF00';

    case Blue = '#0000FF';

    // @mago-format-ignore-start
    case   YELLOW   =   '#FFFF00';
    case   PURPLE   =   '#800080';
    // @mago-format-ignore-end

    // @mago-format-ignore-next
    public function   hex()   :   string   {   return $this->value;   }
}

enum Priority: int
{
    // @mago-format-ignore-next
    case   LOW   =   1;
    case Medium = 2;

    // @mago-format-ignore-next
    case   HIGH   =   3;
}
