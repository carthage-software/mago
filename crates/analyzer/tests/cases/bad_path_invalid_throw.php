<?php

/**
 * @mago-expect analysis:invalid-throw
 */
function testInvalidThrow(): void
{
    throw 'not an exception';
}
