<?php

class Foo
{
    // @mago-format-ignore-next
    public    string    $hookedProp   {
        get   {   return $this->hookedProp;   }
        set   {   $this->hookedProp = $value;   }
    }

    public string $normalHooked {
        get {
            return $this->normalHooked;
        }
    }
}
