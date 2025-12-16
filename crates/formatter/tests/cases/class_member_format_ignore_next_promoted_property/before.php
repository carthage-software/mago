<?php

class Foo
{
    // @mago-format-ignore-next
    public function   __construct(
        public    string    $name   =   'default',
        private    int    $age   =   0,
        protected    array    $data   =   []
    )   {   }
}
