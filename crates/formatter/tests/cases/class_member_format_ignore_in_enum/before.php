<?php

enum MyEnum: int
{
    // @mago-format-ignore-start
    case   A   =   1;
    case   B   =   2;
    case   C   =   3;
    // @mago-format-ignore-end

    case D = 4;

    // @mago-format-ignore-next
    public function   label()   :   string   {   return match($this)   {   self::A => 'A',   self::B => 'B',   self::C => 'C',   self::D => 'D',   };   }

    public function value(): int
    {
        return $this->value;
    }
}
