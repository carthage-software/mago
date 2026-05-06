<?php

interface A {}

interface B extends A {}

/**
 * @require-implements A
 */
trait C {}

/**
 * @require-implements B
 */
trait D
{
    use C;
}
