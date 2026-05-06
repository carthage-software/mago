<?php

class PropertyMagicConstant
{
    public string $foo {
        get => $this->{__PROPERTY__};
    }

    public string $bar {
        set {
            $this->{__PROPERTY__} = strtoupper($value);
        }
    }

    public string $baz {
        get => __PROPERTY__;
        set {
            $this->{__PROPERTY__} = strtolower($value);
        }
    }
}
