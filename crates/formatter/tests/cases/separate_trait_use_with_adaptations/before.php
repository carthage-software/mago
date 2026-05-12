<?php

class ClassName
{
    use FirstTrait, SecondTrait {
        FirstTrait::doStuff insteadof SecondTrait;
    }
}
